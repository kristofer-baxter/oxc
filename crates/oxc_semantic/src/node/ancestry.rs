use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    ptr::{self, NonNull},
    slice,
};

use oxc_allocator::{Address, GetAddress};
use oxc_ast::AstKind;
use oxc_syntax::node::NodeId;

const INITIAL_CAPACITY: u32 = 64;

/// Stack of ancestors of the node currently being visited.
///
/// This is one of the three independent pieces of state the builder maintains
/// while traversing (the other two being the standalone `NodeId` counter and
/// the optional full [`AstNodes`] store). It does **not** depend on the full
/// node store, so it is available even when that store is not retained.
///
/// It is the *only* ancestry information the syntax checker reads, and exposes
/// an **upward-walk-only** API: you can look at the current node and walk up
/// towards the root. There is no downward/sideways traversal.
///
/// Stored as a single Struct-of-Arrays allocation (`[AstKind<'a>; cap]` followed
/// by `[NodeId; cap + 1]` with `NodeId::ROOT` sentinel at slot 0) so `Default`
/// performs a single heap allocation and `push`, `pop`, and `parent_node_id`
/// share one `len`/`cap` with branchless parent-ID restoration.
///
/// [`AstNodes`]: super::AstNodes
pub struct AncestryStack<'a> {
    /// Base pointer to `[AstKind<'a>; cap]` (also the allocation base).
    kinds: NonNull<AstKind<'a>>,
    /// Pointer to `node_ids_base + 1`, where `node_ids_base[0] == NodeId::ROOT`
    /// is the sentinel returned when popping the root node (`Program`).
    node_ids: NonNull<NodeId>,
    len: u32,
    cap: u32,
}

// SAFETY: `AncestryStack` uniquely owns its allocation and its elements are `Send`.
unsafe impl<'a> Send for AncestryStack<'a> where AstKind<'a>: Send {}
// SAFETY: `AncestryStack` uniquely owns its allocation and its elements are `Sync`.
unsafe impl<'a> Sync for AncestryStack<'a> where AstKind<'a>: Sync {}

/// Iterator walking up the ancestry stack, yielding [`AstKind`]s.
type AncestorKinds<'s, 'a> = std::iter::Copied<std::iter::Rev<std::slice::Iter<'s, AstKind<'a>>>>;

impl Default for AncestryStack<'_> {
    fn default() -> Self {
        Self::with_capacity(INITIAL_CAPACITY)
    }
}

impl Drop for AncestryStack<'_> {
    fn drop(&mut self) {
        if self.cap > 0 {
            let layout = Self::compute_layout(self.cap as usize);
            // SAFETY: `self.kinds` was allocated with `compute_layout(self.cap)`.
            unsafe {
                dealloc(self.kinds.as_ptr().cast::<u8>(), layout);
            }
        }
    }
}

impl<'a> AncestryStack<'a> {
    fn compute_layout(cap: usize) -> Layout {
        let kinds_bytes = cap.checked_mul(size_of::<AstKind<'a>>()).expect("capacity overflow");
        let node_ids_bytes = (cap + 1).checked_mul(size_of::<NodeId>()).expect("capacity overflow");
        let total = kinds_bytes.checked_add(node_ids_bytes).expect("capacity overflow");
        Layout::from_size_align(total, align_of::<AstKind<'a>>()).expect("invalid layout")
    }

    #[expect(clippy::cast_ptr_alignment)]
    fn with_capacity(cap: u32) -> Self {
        debug_assert!(cap > 0);
        let layout = Self::compute_layout(cap as usize);
        // SAFETY: `layout` has non-zero size because `cap > 0` and alignment `align_of::<AstKind<'a>>()`.
        unsafe {
            let raw = alloc(layout);
            if raw.is_null() {
                handle_alloc_error(layout);
            }
            let kinds = NonNull::new_unchecked(raw.cast::<AstKind<'a>>());
            let node_ids_base = kinds.as_ptr().add(cap as usize).cast::<NodeId>();
            node_ids_base.write(NodeId::ROOT);
            let node_ids = NonNull::new_unchecked(node_ids_base.add(1));
            Self { kinds, node_ids, len: 0, cap }
        }
    }

    #[cold]
    #[inline(never)]
    #[expect(clippy::cast_ptr_alignment)]
    fn grow(&mut self) {
        let old_cap = self.cap as usize;
        let new_cap = old_cap.checked_mul(2).expect("capacity overflow");
        let new_cap_u32 = u32::try_from(new_cap).expect("capacity overflow");
        let new_layout = Self::compute_layout(new_cap);
        let len = self.len as usize;
        // SAFETY: Copy existing `kinds` (`len` elements) and `node_ids_base` (`len + 1` elements,
        // including the `NodeId::ROOT` sentinel at index 0) into the new allocation, then free the old one.
        unsafe {
            let raw = alloc(new_layout);
            if raw.is_null() {
                handle_alloc_error(new_layout);
            }
            let new_kinds = NonNull::new_unchecked(raw.cast::<AstKind<'a>>());
            let new_node_ids_base = new_kinds.as_ptr().add(new_cap).cast::<NodeId>();
            ptr::copy_nonoverlapping(self.kinds.as_ptr(), new_kinds.as_ptr(), len);
            let old_node_ids_base = self.node_ids.as_ptr().sub(1);
            ptr::copy_nonoverlapping(old_node_ids_base, new_node_ids_base, len + 1);
            let old_layout = Self::compute_layout(old_cap);
            dealloc(self.kinds.as_ptr().cast::<u8>(), old_layout);
            self.kinds = new_kinds;
            self.node_ids = NonNull::new_unchecked(new_node_ids_base.add(1));
            self.cap = new_cap_u32;
        }
    }

    #[inline]
    fn kinds_slice(&self) -> &[AstKind<'a>] {
        // SAFETY: `0..self.len` elements in `self.kinds` have been initialized.
        unsafe { slice::from_raw_parts(self.kinds.as_ptr(), self.len as usize) }
    }

    #[inline]
    fn node_ids_slice(&self) -> &[NodeId] {
        // SAFETY: `0..self.len` elements in `self.node_ids` have been initialized.
        unsafe { slice::from_raw_parts(self.node_ids.as_ptr(), self.len as usize) }
    }

    /// Push the node currently being entered onto the stack.
    #[inline]
    pub(crate) fn push(&mut self, kind: AstKind<'a>, node_id: NodeId) {
        if self.len == self.cap {
            self.grow();
        }
        let idx = self.len as usize;
        // SAFETY: `idx < self.cap` is guaranteed after the capacity check above.
        unsafe {
            self.kinds.as_ptr().add(idx).write(kind);
            self.node_ids.as_ptr().add(idx).write(node_id);
        }
        self.len += 1;
    }

    /// Pop the current node when leaving it, returning the new current [`NodeId`].
    #[inline]
    pub(crate) fn pop(&mut self) -> NodeId {
        debug_assert!(self.len > 0);
        let new_len = self.len - 1;
        self.len = new_len;
        // SAFETY: `self.node_ids` points to `node_ids_base + 1`, where `node_ids_base[0] == NodeId::ROOT`.
        // So `sub(1).add(new_len as usize)` reads `node_ids_base[new_len]`, which is `NodeId::ROOT`
        // when `new_len == 0` and the parent's `NodeId` when `new_len >= 1`.
        unsafe { *self.node_ids.as_ptr().sub(1).add(new_len as usize) }
    }

    /// [`NodeId`] of the parent of the node currently being visited, or
    /// [`NodeId::ROOT`] when the current node is the root (`Program`).
    #[inline]
    pub(crate) fn parent_node_id(&self) -> NodeId {
        debug_assert!(self.len > 0);
        // SAFETY: `self.len >= 1`, and `node_ids_base[0] == NodeId::ROOT`.
        // `sub(1).add((self.len - 1) as usize)` reads `node_ids_base[self.len - 1]`, which is `NodeId::ROOT`
        // when `self.len == 1` (`Program`) and the parent's `NodeId` when `self.len >= 2`.
        unsafe { *self.node_ids.as_ptr().sub(1).add((self.len - 1) as usize) }
    }

    /// [`AstKind`] of the node currently being visited (top of the stack).
    ///
    /// # Panics
    /// Panics if the stack is empty (i.e. not inside a node visit).
    #[inline]
    pub fn current_kind(&self) -> AstKind<'a> {
        *self.kinds_slice().last().expect("ancestry stack is empty")
    }

    /// [`Address`] of the node currently being visited.
    #[inline]
    pub fn current_address(&self) -> Address {
        self.current_kind().address()
    }

    /// [`AstKind`] of the parent of the node currently being visited.
    ///
    /// Returns the current node's own kind when the current node is the root
    /// (`Program`), as `Program` is its own parent.
    #[inline]
    pub fn parent_kind(&self) -> AstKind<'a> {
        let stack = self.kinds_slice();
        // `Program` is its own parent.
        let parent_index = stack.len().saturating_sub(2);
        stack[parent_index]
    }

    /// Walk up the stack, yielding each ancestor's [`AstKind`].
    ///
    /// The first kind produced is the parent of the current node; the last is
    /// `Program`.
    #[inline]
    pub fn ancestor_kinds(&self) -> AncestorKinds<'_, 'a> {
        // Skip the current node (last element), walk towards the root.
        let stack = self.kinds_slice();
        let upto = stack.len().saturating_sub(1);
        stack[..upto].iter().rev().copied()
    }

    /// Walk up the stack starting *at* a chosen node (inclusive), yielding each
    /// [`AstKind`].
    ///
    /// `from` selects the start node:
    /// - `None` starts at the current node (so the first kind is the current
    ///   node, then its ancestors).
    /// - `Some(node_id)` starts at `node_id`, which must be on the current path
    ///   (an ancestor of, or equal to, the node being visited). If not found,
    ///   the returned iterator is empty.
    #[inline]
    pub fn ancestor_kinds_from(&self, from: Option<NodeId>) -> AncestorKinds<'_, 'a> {
        let stack = self.kinds_slice();
        let end = match from {
            None => stack.len(),
            Some(node_id) => {
                self.node_ids_slice().iter().rposition(|&id| id == node_id).map_or(0, |i| i + 1)
            }
        };
        stack[..end].iter().rev().copied()
    }

    /// Find the node with the given [`NodeId`] by walking up the current path,
    /// returning its [`AstKind`].
    ///
    /// Used by checks that resolve `scope -> node` via
    /// `Scoping::get_node_id(scope)` (and similar `class -> node` lookups). The
    /// target is always an ancestor of the node being visited.
    ///
    /// # Panics
    /// Panics if no ancestor has the given `node_id`.
    #[inline]
    pub fn find_kind_by_node_id(&self, node_id: NodeId) -> AstKind<'a> {
        let idx = self
            .node_ids_slice()
            .iter()
            .rposition(|&id| id == node_id)
            .expect("node id not found on the ancestry stack");
        self.kinds_slice()[idx]
    }
}

/// Upward-walking ancestry view used by the syntax checker and binder.
///
/// Dispatches between two backends so the checker/binder code is written once:
///
/// - [`Ancestry::Nodes`] — when the full [`AstNodes`] store is built (linter,
///   mangler), ancestry is read from the store via `current_node_id`. No
///   separate stack is maintained, so this path has no per-node overhead.
/// - [`Ancestry::Stack`] — when the store is *not* built (compiler pipeline),
///   ancestry is read from the [`AncestryStack`] maintained during traversal.
///
/// Either way the surface is upward-walk-only and never exposes random access.
///
/// [`AstNodes`]: super::AstNodes
#[derive(Clone, Copy)]
pub enum Ancestry<'a, 'n> {
    /// Read ancestry from the full node store.
    Nodes { nodes: &'n super::AstNodes<'a>, current_node_id: NodeId },
    /// Read ancestry from the standalone stack.
    Stack(&'n AncestryStack<'a>),
}

impl<'a, 'n> Ancestry<'a, 'n> {
    /// [`AstKind`] of the node currently being visited.
    #[inline]
    pub fn current_kind(self) -> AstKind<'a> {
        match self {
            Ancestry::Nodes { nodes, current_node_id } => nodes.kind(current_node_id),
            Ancestry::Stack(stack) => stack.current_kind(),
        }
    }

    /// [`Address`] of the node currently being visited.
    #[inline]
    pub fn current_address(self) -> Address {
        self.current_kind().address()
    }

    /// [`AstKind`] of the parent of the node currently being visited.
    #[inline]
    pub fn parent_kind(self) -> AstKind<'a> {
        match self {
            Ancestry::Nodes { nodes, current_node_id } => nodes.parent_kind(current_node_id),
            Ancestry::Stack(stack) => stack.parent_kind(),
        }
    }

    /// [`NodeId`] of the parent of the node currently being visited.
    #[inline]
    pub fn parent_node_id(self) -> NodeId {
        match self {
            Ancestry::Nodes { nodes, current_node_id } => nodes.parent_id(current_node_id),
            Ancestry::Stack(stack) => stack.parent_node_id(),
        }
    }

    /// Walk up, yielding each ancestor's [`AstKind`] (parent first, `Program`
    /// last).
    #[inline]
    pub fn ancestor_kinds(self) -> impl Iterator<Item = AstKind<'a>> + Clone + 'n {
        match self {
            Ancestry::Nodes { nodes, current_node_id } => {
                itertools::Either::Left(nodes.ancestor_kinds(current_node_id))
            }
            Ancestry::Stack(stack) => itertools::Either::Right(stack.ancestor_kinds()),
        }
    }

    /// Walk up starting *at* a chosen node (inclusive). `None` starts at the
    /// current node; `Some(node_id)` starts at `node_id` (which must be on the
    /// current path).
    #[inline]
    pub fn ancestor_kinds_from(
        self,
        from: Option<NodeId>,
    ) -> impl Iterator<Item = AstKind<'a>> + Clone + 'n {
        match self {
            Ancestry::Nodes { nodes, current_node_id } => {
                let start = from.unwrap_or(current_node_id);
                itertools::Either::Left(
                    std::iter::once(nodes.kind(start)).chain(nodes.ancestor_kinds(start)),
                )
            }
            Ancestry::Stack(stack) => itertools::Either::Right(stack.ancestor_kinds_from(from)),
        }
    }

    /// Find the node with the given [`NodeId`] and return its [`AstKind`]. The
    /// target is always an ancestor of the node being visited.
    #[inline]
    pub fn find_kind_by_node_id(self, node_id: NodeId) -> AstKind<'a> {
        match self {
            Ancestry::Nodes { nodes, .. } => nodes.kind(node_id),
            Ancestry::Stack(stack) => stack.find_kind_by_node_id(node_id),
        }
    }
}
