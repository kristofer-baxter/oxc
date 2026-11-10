//! Minifier-owned symbol data, grouped by storage lifetime and density.

use rustc_hash::{FxHashMap, FxHashSet};

use std::cell::Cell;

use oxc_allocator::Allocator;
use oxc_ecmascript::constant_evaluation::ConstantValue;
use oxc_index::IndexVec;
use oxc_semantic::Scoping;
use oxc_span::SourceType;
use oxc_str::CompactStr;
use oxc_syntax::symbol::SymbolId;

use crate::{
    CompressOptions,
    symbol_liveness::SymbolLiveness,
    symbol_metadata::{FunctionSummary, MemberWriteEffect, PersistentSymbolMetadata},
    symbol_value::{ReferenceCounts, SymbolValue, SymbolValueDigest},
};

#[derive(Clone, Debug, PartialEq)]
pub enum TrivialFunction<'a> {
    ZeroParamConstant(ConstantValue<'a>),
    OneParamConstant(ConstantValue<'a>),
    Identity1Param,
}

type MemberConstants<'a> = (u32, Option<u32>, FxHashMap<CompactStr, ConstantValue<'a>>);

/// Symbol-indexed data owned by the minifier.
///
/// The fields stay separate because they have different storage and reset
/// contracts:
/// - `values` is dense scratch rebuilt for each peephole pass;
/// - `persistent` is sparse metadata retained across passes;
/// - `liveness` is stable observability metadata plus optional graph results.
pub struct SymbolState<'a> {
    values: IndexVec<SymbolId, Option<SymbolValue<'a>>>,
    populated_symbols: Vec<SymbolId>,
    cached_ref_counts: IndexVec<SymbolId, Cell<Option<ReferenceCounts>>>,
    prev_digests: IndexVec<SymbolId, Option<SymbolValueDigest>>,
    persistent: FxHashMap<SymbolId, PersistentSymbolMetadata>,
    liveness: Option<SymbolLiveness<'a>>,
    pub(crate) member_constants: FxHashMap<SymbolId, MemberConstants<'a>>,
    pub(crate) enum_iife_symbols: FxHashSet<SymbolId>,
    pub(crate) verified_enum_iifes: FxHashMap<
        SymbolId,
        (Option<oxc_syntax::reference::ReferenceId>, Option<oxc_syntax::reference::ReferenceId>),
    >,
    pub(crate) trivial_functions: FxHashMap<SymbolId, (u32, Option<u32>, TrivialFunction<'a>)>,
    pub(crate) cached_allow_enum_iife: Option<bool>,
}

impl<'a> SymbolState<'a> {
    pub fn new(
        source_type: SourceType,
        options: &CompressOptions,
        scoping: &Scoping,
        allocator: &'a Allocator,
    ) -> Self {
        let len = scoping.symbols_len();
        let mut values = IndexVec::with_capacity(len);
        values.resize_with(len, || None);
        let mut cached_ref_counts = IndexVec::with_capacity(len);
        cached_ref_counts.resize_with(len, || Cell::new(None));
        let mut prev_digests = IndexVec::with_capacity(len);
        prev_digests.resize_with(len, || None);
        Self {
            values,
            populated_symbols: Vec::with_capacity(len / 2),
            cached_ref_counts,
            prev_digests,
            persistent: FxHashMap::default(),
            liveness: SymbolLiveness::new_if_enabled(source_type, options, scoping, allocator),
            member_constants: FxHashMap::default(),
            enum_iife_symbols: FxHashSet::default(),
            verified_enum_iifes: FxHashMap::default(),
            trivial_functions: FxHashMap::default(),
            cached_allow_enum_iife: None,
        }
    }

    /// Clear values without releasing the buffer so the next pass stays on
    /// the indexed-write fast path.
    pub fn reset_values(&mut self) {
        for symbol_id in self.populated_symbols.drain(..) {
            self.values[symbol_id] = None;
        }
    }

    #[inline]
    pub fn init_value(
        &mut self,
        symbol_id: SymbolId,
        value: SymbolValue<'a>,
    ) -> &mut SymbolValue<'a> {
        let slot = &mut self.values[symbol_id];
        if slot.is_none() {
            self.populated_symbols.push(symbol_id);
        }
        slot.insert(value)
    }

    #[inline]
    pub fn value(&self, symbol_id: SymbolId) -> Option<&SymbolValue<'a>> {
        self.values.get(symbol_id)?.as_ref()
    }

    #[inline]
    pub fn has_cached_ref_counts(&self, symbol_id: SymbolId) -> bool {
        self.cached_ref_counts[symbol_id].get().is_some()
    }

    #[inline]
    pub fn reference_counts(&self, symbol_id: SymbolId, scoping: &Scoping) -> ReferenceCounts {
        if let Some(counts) = self.cached_ref_counts[symbol_id].get() {
            return counts;
        }
        let mut counts = ReferenceCounts::default();
        for reference in scoping.get_resolved_references(symbol_id) {
            counts.record(reference.flags());
        }
        self.cached_ref_counts[symbol_id].set(Some(counts));
        counts
    }

    #[inline]
    pub fn symbol_is_mutated(&self, symbol_id: SymbolId, scoping: &Scoping) -> bool {
        if scoping.symbol_flags(symbol_id).is_const_variable() {
            return false;
        }
        if let Some(sv) = self.value(symbol_id) {
            return sv.references.has_writes();
        }
        self.reference_counts(symbol_id, scoping).has_writes()
    }

    #[inline]
    pub fn invalidate_symbol(&self, symbol_id: SymbolId) {
        self.cached_ref_counts[symbol_id].set(None);
    }

    #[inline]
    pub fn update_digest(&mut self, symbol_id: SymbolId, digest: SymbolValueDigest) -> bool {
        let prev = self.prev_digests[symbol_id].replace(digest);
        prev != Some(digest)
    }

    pub fn set_function_summary(&mut self, symbol_id: SymbolId, summary: FunctionSummary) -> bool {
        let entry = self.persistent.entry(symbol_id).or_default();
        if entry.function_summary() == summary {
            return false;
        }
        entry.set_function_summary(summary);
        true
    }

    pub fn clear_function_summary(&mut self, symbol_id: SymbolId) -> bool {
        if let Some(metadata) = self.persistent.get_mut(&symbol_id)
            && metadata.function_summary() != FunctionSummary::Unknown
        {
            metadata.set_function_summary(FunctionSummary::Unknown);
            return true;
        }
        false
    }

    pub fn function_summary(&self, symbol_id: SymbolId) -> FunctionSummary {
        self.persistent
            .get(&symbol_id)
            .map_or(FunctionSummary::Unknown, PersistentSymbolMetadata::function_summary)
    }

    pub fn record_member_write_effect(&mut self, symbol_id: SymbolId, effect: MemberWriteEffect) {
        self.persistent.entry(symbol_id).or_default().record_member_write_effect(effect);
    }

    pub fn member_write_effect(&self, symbol_id: SymbolId) -> MemberWriteEffect {
        self.persistent
            .get(&symbol_id)
            .map_or(MemberWriteEffect::None, PersistentSymbolMetadata::member_write_effect)
    }

    /// Whether runtime semantics have an implicit observation channel that
    /// remains even if every resolved reference disappears from the current
    /// AST. Returns `false` when liveness state is absent, which is only the
    /// case when Normalize recorded no such binding.
    pub fn is_implicitly_observable(&self, symbol_id: SymbolId) -> bool {
        self.liveness.as_ref().is_some_and(|liveness| liveness.is_implicitly_observable(symbol_id))
    }

    /// Whether post-flush graph analysis proved a function declaration
    /// unreachable from executing code. Returns `false` when liveness state is
    /// absent because deadness was not proved.
    pub fn function_is_dead(&self, symbol_id: SymbolId) -> bool {
        self.liveness.as_ref().is_some_and(|liveness| liveness.function_is_dead(symbol_id))
    }

    pub fn liveness(&self) -> Option<&SymbolLiveness<'a>> {
        self.liveness.as_ref()
    }

    pub fn liveness_mut(&mut self) -> Option<&mut SymbolLiveness<'a>> {
        self.liveness.as_mut()
    }

    pub fn ensure_liveness(
        &mut self,
        create: impl FnOnce() -> SymbolLiveness<'a>,
    ) -> &mut SymbolLiveness<'a> {
        self.liveness.get_or_insert_with(create)
    }
}
