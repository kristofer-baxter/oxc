use oxc_allocator::{Allocator, BitSet};
use oxc_ast::ast::{Function, Program};
use oxc_ast_visit::{Visit, walk};
use oxc_data_structures::stack::NonEmptyStack;
use oxc_semantic::Scoping;
use oxc_span::SourceType;
use oxc_str::{Ident, IdentHashSet};
use oxc_syntax::{
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};
use rustc_hash::FxHashSet;

use crate::{CompressOptions, symbol_liveness::SymbolLiveness, symbol_state::SymbolState};

/// Compression pipeline selected for this run.
#[derive(Clone, Copy)]
pub enum CompressionMode {
    /// Remove dead code and apply size-reducing transformations.
    Full,
    /// Remove dead and unused code without otherwise shrinking the output.
    /// This still runs the constant folds needed to expose removable branches,
    /// but skips transforms whose only purpose is smaller syntax. Rolldown uses
    /// this pipeline for tree-shaking with or without output minification.
    TreeShakeOnly,
}

/// Immutable configuration shared by all compression passes in one run.
struct CompressionConfig {
    source_type: SourceType,
    options: CompressOptions,
    mode: CompressionMode,
}

/// Changes accumulated between two pass-completion boundaries. Live from
/// `MinifierState::new` so the pre-loop `Normalize` pass records drops through
/// the same typed helpers as the peephole loop; consumed and reset or resized
/// by [`crate::compression_pass`] after Normalize and every peephole pass.
pub struct PassChanges<'a> {
    /// Whether the completed pass changed facts or AST shape in a way that
    /// requires another peephole traversal. This signal does not itself force
    /// liveness analysis; removed candidate references and dropped direct eval
    /// calls continue to gate that work independently.
    revisit_requested: bool,

    /// `ReferenceId`s whose AST node has been removed and not re-installed
    /// in any later mutation this pass.
    ///
    /// Arena-allocated bitset sized to the program's `references_len()` at
    /// construction / the previous flush. A `BitSet` (rather than an
    /// `FxHashSet`) keeps the per-ident cost on the
    /// `DroppedSubtreeCollector` hot path to
    /// a direct array store instead of a hash + heap insert.
    ///
    /// INVARIANT (the "capacity guard", relied on by `DroppedSubtreeCollector`,
    /// `Scoping::retain_resolved_references_excluding`, and the over-prune
    /// debug assert): references minted MID-pass have indices beyond the
    /// bitset's capacity and are treated as live everywhere — never marked,
    /// never excluded. Conservative: such a reference stays in its symbol's
    /// list until callers rebuild scoping (a missed optimization, never a
    /// correctness issue). `Normalize` mints no references, so a capacity
    /// taken at construction is exact for the first pass.
    pub(crate) removed_references: BitSet<'a>,

    /// At least one direct `eval(...)` call was dropped this pass. Gates
    /// the small `LiveDirectEvalCollector` walk at flush time.
    pub(crate) direct_eval_dropped: bool,

    /// Function and arrow scopes whose enclosing node was dropped or dissolved
    /// (such as an inlined or expanded IIFE). Moved references retain their
    /// original scope id, so dirty propagation treats dissolved function scopes
    /// as transparent and continues up to the surviving enclosing function.
    pub(crate) dissolved_scopes: BitSet<'a>,
}

impl<'a> PassChanges<'a> {
    pub fn new(references_len: usize, scopes_len: usize, allocator: &'a Allocator) -> Self {
        Self {
            revisit_requested: false,
            removed_references: BitSet::new_in(references_len, allocator),
            direct_eval_dropped: false,
            dissolved_scopes: BitSet::new_in(scopes_len, allocator),
        }
    }
}

/// State associated with one enclosing function body or the program root.
pub struct BodyFrame {
    /// Semantic scope containing the body's top-level statements.
    pub scope_id: ScopeId,

    /// Whether a preceding statement could observe a later hoisted variable
    /// before its initializer runs. The program root also starts unsafe when a
    /// module loader could expose its bindings through a cycle.
    pub hoisted_var_inlining_unsafe: bool,

    /// Remaining body-level variable declarations in this body yet to be
    /// visited. Once `0`, later statements cannot precede any body-level
    /// variable declarator.
    pub last_var_decl_end: u32,

    /// Source offset after the first unconditional top-level `super()` call,
    /// used to distinguish initialized and uninitialized derived-constructor
    /// `this` accesses.
    pub this_initialized_at: Option<u32>,
}

/// Change-tracking frame for an enclosing function scope (including formal
/// parameters and body) or the program root.
#[derive(Clone, Copy)]
pub struct FnScopeFrame {
    pub scope_id: ScopeId,
    pub saved_total_changes: u64,
    pub saved_child_changes: u64,
}

pub struct MinifierState<'a> {
    /// Source semantics, compression options, and pipeline selection fixed for
    /// the lifetime of this run.
    config: CompressionConfig,

    /// Dense per-pass values, sparse persistent metadata, and optional
    /// reachability data indexed by semantic symbols.
    pub(crate) symbols: SymbolState<'a>,

    /// Private-name usage scoped by nested classes. This tracks `#name`
    /// strings, not semantic `SymbolId`s, so it deliberately stays outside
    /// `SymbolState`.
    pub private_member_usage: PrivateMemberUsageStack<'a>,

    /// One frame per enclosing function body (program root at the bottom).
    /// Pushed by `enter_function_body` and popped by `exit_function_body`.
    pub body_frames: NonEmptyStack<BodyFrame>,

    /// One frame per enclosing function scope (program root at the bottom).
    /// Covers both formal parameters and the function body.
    pub fn_scope_frames: NonEmptyStack<FnScopeFrame>,

    /// Per-pass change accumulator populated by typed mutation helpers and
    /// consumed by `compression_pass` after Normalize and every peephole pass.
    pub(crate) pass_changes: PassChanges<'a>,

    /// Reusable scratch buffer for `PeepholeOptimizations::minimize_statements`
    /// so statement lists can be minimized in place without reallocating in the
    /// bump arena on every pass.
    pub(crate) stmts_scratch: oxc_allocator::Vec<'a, oxc_ast::ast::Statement<'a>>,

    pub(crate) pass_index: u8,
    pub(crate) cascading_change_counter: u64,
    pub(crate) child_fn_cascading_changes: u64,
    pub(crate) dirty_all_scopes: bool,
    pub(crate) seen_scopes: BitSet<'a>,
    pub(crate) dirty_scopes: BitSet<'a>,
    pub(crate) next_dirty_scopes: BitSet<'a>,
    pub(crate) own_dirty_scopes: BitSet<'a>,
    pub(crate) next_own_dirty_scopes: BitSet<'a>,

    /// Bindings that can be written without a resolved write reference: a
    /// function-scoped binding that a sloppy-mode block function declaration of
    /// the same name can overwrite (Annex B.3.3, see
    /// `annex_b_observable_bindings`), a lexical binding whose presence blocks
    /// that overwrite, and a CommonJS root binding that the
    /// wrapper's mapped `arguments` aliases (see `commonjs_wrapper_bindings`).
    pub(crate) hidden_writes: FxHashSet<SymbolId>,
}

/// Parameters of the function that wraps a CommonJS module body.
pub const COMMONJS_WRAPPER_PARAMETERS: [&str; 5] =
    ["exports", "require", "module", "__filename", "__dirname"];

/// Root bindings of a CommonJS body that redeclare a wrapper parameter. The
/// wrapper is a sloppy function, so its mapped `arguments` reads and writes
/// them with no resolved reference.
pub fn commonjs_wrapper_bindings(
    source_type: SourceType,
    scoping: &Scoping,
) -> impl Iterator<Item = SymbolId> + '_ {
    let root_scope_id = scoping.root_scope_id();
    COMMONJS_WRAPPER_PARAMETERS
        .into_iter()
        .filter(move |_| source_type.is_commonjs())
        .filter_map(move |name| scoping.get_binding(root_scope_id, name.into()))
}

/// Protect implicit Annex B writes and the bindings that block them. Use the
/// declaration's scope because semantic hoisting can move its symbol past a
/// lexical barrier.
fn annex_b_observable_bindings(program: &Program<'_>, scoping: &Scoping) -> FxHashSet<SymbolId> {
    struct Collector<'s> {
        scoping: &'s Scoping,
        bindings: FxHashSet<SymbolId>,
    }

    impl<'a> Visit<'a> for Collector<'_> {
        fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
            if function.is_function_declaration()
                && !function.r#async
                && !function.generator
                && let Some(id) = &function.id
                && let Some(scope_id) = self.scoping.scope_parent_id(function.scope_id())
                && !self.scoping.scope_flags(scope_id).is_var()
                && !self.scoping.scope_flags(scope_id).is_strict_mode()
            {
                if let Some(binding) = self.scoping.get_binding(scope_id, id.name)
                    && self.scoping.symbol_flags(binding).intersects(
                        SymbolFlags::BlockScopedVariable
                            | SymbolFlags::Class
                            | SymbolFlags::CatchVariable,
                    )
                {
                    self.bindings.insert(binding);
                }
                for ancestor in self.scoping.scope_ancestors(scope_id).skip(1) {
                    if let Some(binding) = self.scoping.get_binding(ancestor, id.name) {
                        self.bindings.insert(binding);
                    }
                    if self.scoping.scope_flags(ancestor).is_var() {
                        break;
                    }
                }
            }
            walk::walk_function(self, function, flags);
        }
    }

    let mut collector = Collector { scoping, bindings: FxHashSet::default() };
    collector.visit_program(program);
    collector.bindings
}

impl<'a> MinifierState<'a> {
    pub(crate) fn new(
        program: &Program<'_>,
        options: CompressOptions,
        mode: CompressionMode,
        scoping: &Scoping,
        allocator: &'a Allocator,
    ) -> Self {
        let source_type = program.source_type;
        let mut hidden_writes = if source_type.is_strict() {
            FxHashSet::default()
        } else {
            annex_b_observable_bindings(program, scoping)
        };
        hidden_writes.extend(commonjs_wrapper_bindings(source_type, scoping));
        let mut symbols = SymbolState::new(source_type, &options, scoping, allocator);
        if !hidden_writes.is_empty() {
            let liveness =
                symbols.ensure_liveness(|| SymbolLiveness::new(source_type, scoping, allocator));
            for &binding in &hidden_writes {
                let flags = scoping.symbol_flags(binding);
                if flags.intersects(SymbolFlags::BlockScopedVariable | SymbolFlags::Class)
                    || (flags.contains(SymbolFlags::CatchVariable)
                        && !flags.contains(SymbolFlags::FunctionScopedVariable))
                {
                    liveness.mark_implicitly_observable(binding);
                }
            }
        }
        Self {
            config: CompressionConfig { source_type, options, mode },
            symbols,
            private_member_usage: PrivateMemberUsageStack::new(),
            body_frames: NonEmptyStack::new(BodyFrame {
                scope_id: scoping.root_scope_id(),
                hoisted_var_inlining_unsafe: false,
                last_var_decl_end: 0,
                this_initialized_at: None,
            }),
            fn_scope_frames: NonEmptyStack::new(FnScopeFrame {
                scope_id: scoping.root_scope_id(),
                saved_total_changes: 0,
                saved_child_changes: 0,
            }),
            pass_changes: PassChanges::new(
                scoping.references_len(),
                scoping.scopes_len(),
                allocator,
            ),
            pass_index: 0,
            cascading_change_counter: 0,
            child_fn_cascading_changes: 0,
            dirty_all_scopes: false,
            seen_scopes: BitSet::new_in(scoping.scopes_len(), allocator),
            dirty_scopes: BitSet::new_in(scoping.scopes_len(), allocator),
            next_dirty_scopes: BitSet::new_in(scoping.scopes_len(), allocator),
            own_dirty_scopes: BitSet::new_in(scoping.scopes_len(), allocator),
            next_own_dirty_scopes: BitSet::new_in(scoping.scopes_len(), allocator),
            hidden_writes,
            stmts_scratch: oxc_allocator::Vec::new_in(&allocator),
        }
    }

    /// Whether `Normalize`'s member-write scan should seed persistent metadata,
    /// i.e. whether any consumer is live in this configuration. In full
    /// minify the default-mode write-only property drop reads
    /// hazardous-member state and the shared drop predicate reads possible
    /// prototype mutation. In tree-shake-only mode the default path is disabled
    /// and only the `property_write_side_effects: false` opt-in drop reads the
    /// prototype state, so with the knob left on nothing reads the effects and
    /// seeding is skipped.
    pub(crate) fn should_track_member_write_effects(&self) -> bool {
        !self.is_tree_shake_only() || !self.options().treeshake.property_write_side_effects
    }

    pub(crate) fn options(&self) -> &CompressOptions {
        &self.config.options
    }

    pub(crate) fn source_type(&self) -> SourceType {
        self.config.source_type
    }

    pub(crate) fn is_tree_shake_only(&self) -> bool {
        matches!(self.config.mode, CompressionMode::TreeShakeOnly)
    }

    /// Request another peephole traversal after the current pass completes.
    ///
    /// This is deliberately separate from [`Self::record_ast_change`] for
    /// future fact-only callers such as #24060. Such a caller may enable more
    /// transforms without changing the AST, resolved references, or direct-eval
    /// scope flags.
    pub(crate) fn request_revisit(&mut self) {
        self.pass_changes.revisit_requested = true;
    }

    /// Record an AST change and schedule the traversal needed to consume it.
    /// Typed mutation helpers use this as their AST-change entry point.
    pub(crate) fn record_ast_change(&mut self) {
        self.cascading_change_counter += 1;
        self.request_revisit();
    }

    #[inline]
    pub(crate) fn is_current_body_own_dirty(&self) -> bool {
        let frame = self.fn_scope_frames.last();
        let idx = frame.scope_id.index();
        let own_changed = (self.cascading_change_counter - frame.saved_total_changes)
            > (self.child_fn_cascading_changes - frame.saved_child_changes);
        self.pass_index == 0
            || self.dirty_all_scopes
            || !self.private_member_usage.is_at_root()
            || own_changed
            || idx >= self.own_dirty_scopes.capacity()
            || self.own_dirty_scopes.has_bit(idx)
    }

    #[inline]
    pub(crate) fn enter_root_scope(&mut self, scope_id: ScopeId) {
        *self.fn_scope_frames.last_mut() = FnScopeFrame {
            scope_id,
            saved_total_changes: self.cascading_change_counter,
            saved_child_changes: self.child_fn_cascading_changes,
        };
    }

    #[inline]
    pub(crate) fn exit_root_scope(&mut self) {
        let frame = *self.fn_scope_frames.last();
        let total_delta = self.cascading_change_counter - frame.saved_total_changes;
        let child_delta = self.child_fn_cascading_changes - frame.saved_child_changes;
        if total_delta > 0 {
            // Child simplifications can expose new candidates to the root's definition scan.
            self.mark_scope_dirty_for_next_pass(frame.scope_id);
        }
        if total_delta > child_delta {
            self.mark_scope_own_dirty_for_next_pass(frame.scope_id);
        }
    }

    #[inline]
    pub(crate) fn enter_function_scope(&mut self, scope_id: ScopeId) {
        self.record_seen_scope(scope_id);
        self.fn_scope_frames.push(FnScopeFrame {
            scope_id,
            saved_total_changes: self.cascading_change_counter,
            saved_child_changes: self.child_fn_cascading_changes,
        });
    }

    #[inline]
    pub(crate) fn exit_function_scope(&mut self) {
        let frame = self.fn_scope_frames.pop();
        let total_delta = self.cascading_change_counter - frame.saved_total_changes;
        let child_delta = self.child_fn_cascading_changes - frame.saved_child_changes;
        if total_delta > 0 {
            self.mark_scope_dirty_for_next_pass(frame.scope_id);
            if total_delta > child_delta {
                self.mark_scope_own_dirty_for_next_pass(frame.scope_id);
            }
            self.child_fn_cascading_changes = frame.saved_child_changes + total_delta;
        }
    }

    #[inline]
    pub(crate) fn mark_scope_own_dirty_for_next_pass(&mut self, scope_id: ScopeId) {
        let idx = scope_id.index();
        if idx < self.next_own_dirty_scopes.capacity() {
            self.next_own_dirty_scopes.set_bit(idx);
            self.next_dirty_scopes.set_bit(idx);
        }
    }

    /// Record a leaf/local AST change that is already normal in the current
    /// bottom-up pass, so it requests another pass without forcing the
    /// enclosing function body to be re-walked if nothing else changed.
    pub(crate) fn record_non_cascading_ast_change(&mut self) {
        self.request_revisit();
    }

    #[inline]
    pub(crate) fn should_skip_function_scope(&self, scope_id: ScopeId) -> bool {
        let idx = scope_id.index();
        self.pass_index > 0
            && !self.dirty_all_scopes
            && self.private_member_usage.is_at_root()
            && idx < self.seen_scopes.capacity()
            && self.seen_scopes.has_bit(idx)
            && !self.dirty_scopes.has_bit(idx)
    }

    #[inline]
    pub(crate) fn record_seen_scope(&mut self, scope_id: ScopeId) {
        let idx = scope_id.index();
        if idx < self.seen_scopes.capacity() {
            self.seen_scopes.set_bit(idx);
        }
    }

    #[inline]
    pub(crate) fn mark_scope_dirty_for_next_pass(&mut self, scope_id: ScopeId) {
        let idx = scope_id.index();
        if idx < self.next_dirty_scopes.capacity() {
            self.next_dirty_scopes.set_bit(idx);
        }
    }

    pub(crate) fn mark_scope_and_ancestors_dirty(
        &mut self,
        scope_id: ScopeId,
        scoping: &Scoping,
        current_pass_too: bool,
    ) {
        let cap = self.next_dirty_scopes.capacity();
        let mut mark_own = true;
        for ancestor in scoping.scope_ancestors(scope_id) {
            let idx = ancestor.index();
            if idx >= cap {
                continue;
            }
            if mark_own {
                self.next_own_dirty_scopes.set_bit(idx);
                if current_pass_too {
                    self.own_dirty_scopes.set_bit(idx);
                }
                let dissolved = idx < self.pass_changes.dissolved_scopes.capacity()
                    && self.pass_changes.dissolved_scopes.has_bit(idx);
                if (scoping.scope_flags(ancestor).is_function() && !dissolved)
                    || ancestor == scoping.root_scope_id()
                {
                    mark_own = false;
                }
            }
            if current_pass_too {
                self.dirty_scopes.set_bit(idx);
            }
            if self.next_dirty_scopes.has_bit(idx) && !current_pass_too && !mark_own {
                break;
            }
            self.next_dirty_scopes.set_bit(idx);
        }
    }

    pub(crate) fn mark_symbol_scopes_dirty_current_pass(
        &mut self,
        symbol_id: SymbolId,
        scoping: &Scoping,
    ) {
        let cap = self.dirty_scopes.capacity();
        for scope_id in std::iter::once(scoping.symbol_scope_id(symbol_id)).chain(
            scoping.get_resolved_references(symbol_id).map(oxc_semantic::Reference::scope_id),
        ) {
            let mut mark_own = true;
            for ancestor in scoping.scope_ancestors(scope_id) {
                let idx = ancestor.index();
                if idx >= cap {
                    continue;
                }
                if mark_own {
                    self.own_dirty_scopes.set_bit(idx);
                    let dissolved = idx < self.pass_changes.dissolved_scopes.capacity()
                        && self.pass_changes.dissolved_scopes.has_bit(idx);
                    if (scoping.scope_flags(ancestor).is_function() && !dissolved)
                        || ancestor == scoping.root_scope_id()
                    {
                        mark_own = false;
                    }
                }
                if self.dirty_scopes.has_bit(idx) && !mark_own {
                    break;
                }
                self.dirty_scopes.set_bit(idx);
            }
        }
    }

    pub(crate) fn mark_symbol_scopes_dirty(
        &mut self,
        symbol_id: SymbolId,
        scoping: &Scoping,
        current_pass_too: bool,
    ) {
        self.mark_scope_and_ancestors_dirty(
            scoping.symbol_scope_id(symbol_id),
            scoping,
            current_pass_too,
        );
        for reference in scoping.get_resolved_references(symbol_id) {
            self.mark_scope_and_ancestors_dirty(reference.scope_id(), scoping, current_pass_too);
        }
    }

    pub(crate) fn advance_pass(&mut self) {
        std::mem::swap(&mut self.dirty_scopes, &mut self.next_dirty_scopes);
        self.next_dirty_scopes.clear();
        std::mem::swap(&mut self.own_dirty_scopes, &mut self.next_own_dirty_scopes);
        self.next_own_dirty_scopes.clear();
        self.pass_index = self.pass_index.saturating_add(1);
    }

    /// Return whether the completed pass requested another traversal, then
    /// reset the signal for the next pass.
    pub(crate) fn take_revisit_requested(&mut self) -> bool {
        std::mem::take(&mut self.pass_changes.revisit_requested)
    }

    /// Whether every per-pass change has been consumed.
    pub(crate) fn pass_changes_are_clean(&self) -> bool {
        !self.pass_changes.revisit_requested
            && self.pass_changes.removed_references.is_empty()
            && !self.pass_changes.direct_eval_dropped
    }
}

/// Private member names used in each currently enclosing class.
pub struct PrivateMemberUsageStack<'a> {
    stack: NonEmptyStack<IdentHashSet<'a>>,
}

impl<'a> PrivateMemberUsageStack<'a> {
    pub fn new() -> Self {
        Self { stack: NonEmptyStack::new(IdentHashSet::default()) }
    }

    /// Whether all class scopes have been exited.
    pub fn is_at_root(&self) -> bool {
        self.stack.is_exhausted()
    }

    pub fn enter_class(&mut self) {
        self.stack.push(IdentHashSet::default());
    }

    /// Exit a class and propagate uses of names declared by an outer class.
    pub fn exit_class(&mut self, declared_private_members: impl Iterator<Item = Ident<'a>>) {
        let mut used_private_members = self.stack.pop();
        declared_private_members.for_each(|name| {
            used_private_members.remove(&name);
        });
        self.stack.last_mut().extend(used_private_members);
    }

    pub fn record_use(&mut self, name: Ident<'a>) {
        self.stack.last_mut().insert(name);
    }

    pub fn is_used(&self, name: &Ident<'a>) -> bool {
        self.stack.last().contains(name)
    }
}
