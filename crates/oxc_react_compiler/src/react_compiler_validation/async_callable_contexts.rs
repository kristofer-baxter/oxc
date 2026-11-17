use std::cell::RefCell;
use std::rc::Rc;

use oxc_span::Span;
use rustc_hash::{FxHashMap, FxHashSet};

use super::validate_locals_not_reassigned_after_render::no_alias_callback_parameters;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::visitors::{
    each_instruction_value_lvalue, each_instruction_value_operand, each_terminal_operand,
};
use crate::react_compiler_hir::{
    ArrayElement, ArrayPatternElement, BlockId, HirFunction, IdentifierId, Instruction,
    InstructionId, InstructionKind, InstructionValue, NonLocalBinding, ObjectPropertyKey,
    ObjectPropertyOrSpread, ParamPattern, Pattern, PlaceOrSpread, PrimitiveValue, Terminal,
};

type Captures = FxHashMap<IdentifierId, FxHashSet<IdentifierId>>;
type Properties = FxHashMap<Option<String>, Values>;

#[derive(Clone, PartialEq, Eq, Hash)]
struct Projection {
    object: IdentifierId,
    key: Option<String>,
}

// Parameter member reads retain their source until a caller supplies the object.
// Instruction identities make this graph finite even for cyclic property walks.
type Projections = FxHashMap<IdentifierId, FxHashSet<Projection>>;

#[derive(Clone, PartialEq, Eq)]
struct Arguments {
    elements: Properties,
    length: Option<usize>,
}

impl Default for Arguments {
    fn default() -> Self {
        Self { elements: Properties::default(), length: Some(0) }
    }
}

impl Arguments {
    fn append(&mut self, other: &Self) {
        for (key, values) in &other.elements {
            let key = self.length.and_then(|offset| {
                array_index(key.as_ref()?)?.checked_add(offset).map(|i| i.to_string())
            });
            self.elements.entry(key).or_default().extend(values);
        }
        self.length = self.length.zip(other.length).and_then(|(a, b)| a.checked_add(b));
    }

    fn merge(&mut self, other: &Self) {
        merge_properties(&mut self.elements, &other.elements);
        if self.length != other.length {
            self.length = None;
        }
    }

    /// Values that may land at `index` after spreads are expanded.
    fn at(&self, index: usize) -> Values {
        let mut values = self.elements.get(&Some(index.to_string())).cloned().unwrap_or_default();
        if let Some(unknown) = self.elements.get(&None) {
            values.extend(unknown);
        }
        values
    }

    /// Values that may land at `from` or any later position.
    fn from(&self, from: usize) -> Values {
        let mut values = Values::default();
        for (key, element) in &self.elements {
            if key.as_ref().is_none_or(|key| array_index(key).is_some_and(|key| key >= from)) {
                values.extend(element);
            }
        }
        values
    }

    fn skip(mut self, count: usize) -> Self {
        self.elements = self
            .elements
            .into_iter()
            .filter_map(|(key, value)| {
                let key = match key {
                    Some(key) => Some(array_index(&key)?.checked_sub(count)?.to_string()),
                    None => None,
                };
                Some((key, value))
            })
            .collect();
        self.length = self.length.map(|length| length.saturating_sub(count));
        self
    }
}

/// A captured factory is resolved at its caller, where the live callee and its
/// returned-callable summary are available. Instruction identities bound cycles.
#[derive(Clone, Default, PartialEq, Eq)]
struct DeferredCall {
    callees: FxHashSet<IdentifierId>,
    arguments: Arguments,
}

impl DeferredCall {
    fn merge(&mut self, other: &Self) {
        self.callees.extend(&other.callees);
        self.arguments.merge(&other.arguments);
    }
}

type DeferredCalls = FxHashMap<IdentifierId, DeferredCall>;

/// Calls executed by a function are distinct from its returned values. A caller
/// resolves captured or parameter callees before applying nested async effects.
#[derive(Clone, PartialEq, Eq)]
struct Call {
    callees: Values,
    arguments: Option<Arguments>,
}

/// Non-plain parameters are temporaries followed by prologue binding stores.
/// Declaration locations identify those bindings without including body aliases.
fn parameter_bindings(func: &HirFunction, env: &Environment<'_>) -> FxHashSet<IdentifierId> {
    let parameters: Vec<_> = func
        .params
        .iter()
        .map(|param| match param {
            ParamPattern::Place(place) => place.identifier,
            ParamPattern::Spread(spread) => spread.place.identifier,
        })
        .collect();
    let spans: Vec<_> = parameters.iter().filter_map(|&id| env.identifiers[id].span).collect();
    let mut bindings: FxHashSet<_> = parameters.into_iter().collect();
    for instruction in &func.instructions {
        for place in each_instruction_value_lvalue(&instruction.value) {
            if env.identifiers[place.identifier].span.is_some_and(|span| {
                spans
                    .iter()
                    .any(|parameter| parameter.start <= span.start && span.end <= parameter.end)
            }) {
                bindings.insert(place.identifier);
            }
        }
    }
    // Temporary parameter inputs are observed through their initialized bindings.
    // Counting the whole prologue input would include unused sibling properties.
    for param in &func.params {
        match param {
            ParamPattern::Place(place) if place.span.is_none() => {
                bindings.remove(&place.identifier);
            }
            ParamPattern::Spread(spread) => {
                bindings.remove(&spread.place.identifier);
            }
            _ => {}
        }
    }
    bindings
}

/// Preserve capture provenance through local snapshots, including object
/// projections. Instruction identities keep cyclic CFGs finite.
fn propagate_capture_origins(
    func: &HirFunction,
    origins: &mut Captures,
    stop_at: &FxHashSet<IdentifierId>,
) {
    loop {
        let mut changed = false;
        let mut copy = |from, into| {
            if stop_at.contains(&into) {
                return;
            }
            let values = origins.get(&from).cloned().unwrap_or_default();
            let target = origins.entry(into).or_default();
            let old_len = target.len();
            target.extend(values);
            changed |= old_len != target.len();
        };
        for (_, block) in &func.body.blocks {
            for phi in &block.phis {
                for operand in phi.operands.values() {
                    copy(operand.identifier, phi.place.identifier);
                }
            }
            for &id in &block.instructions {
                let instruction = &func.instructions[id.index()];
                let into = instruction.lvalue.identifier;
                match &instruction.value {
                    InstructionValue::LoadLocal { place, .. }
                    | InstructionValue::LoadContext { place, .. }
                        if !stop_at.contains(&place.identifier) =>
                    {
                        copy(place.identifier, into)
                    }
                    InstructionValue::TypeCastExpression { value, .. } => {
                        copy(value.identifier, into)
                    }
                    InstructionValue::PropertyLoad { object, .. }
                    | InstructionValue::ComputedLoad { object, .. } => {
                        copy(object.identifier, into)
                    }
                    InstructionValue::StoreLocal { lvalue, value, .. }
                    | InstructionValue::StoreContext { lvalue, value, .. } => {
                        copy(value.identifier, lvalue.place.identifier);
                        copy(value.identifier, into);
                    }
                    _ => {}
                }
            }
        }
        if !changed {
            break;
        }
    }
}

/// Track binding reads and member observations separately: a saved object keeps
/// its identity before await, but its members are observed after suspension.
fn infer_capture_phases(
    func: &HirFunction,
    invocations: &FxHashMap<InstructionId, Vec<AsyncInvocation>>,
    member_reads: &FxHashMap<IdentifierId, FxHashSet<String>>,
    parameters: &FxHashSet<IdentifierId>,
) -> (FxHashSet<IdentifierId>, FxHashSet<IdentifierId>, FxHashSet<IdentifierId>) {
    const EAGER: u8 = 1;
    const DEFERRED: u8 = 2;
    let contexts: FxHashSet<_> = func
        .context
        .iter()
        .map(|place| place.identifier)
        .chain(parameters.iter().copied())
        .collect();
    let mut eager = FxHashSet::default();
    let mut deferred = FxHashSet::default();
    let mut snapshots = FxHashSet::default();
    if contexts.is_empty() {
        return (eager, deferred, snapshots);
    }
    let mut phases = FxHashMap::default();
    let mut instruction_phases = FxHashMap::default();
    loop {
        let mut changed = false;
        for (&id, block) in &func.body.blocks {
            let mut phase = if id == func.body.entry { EAGER } else { 0 };
            for predecessor in &block.preds {
                phase |= phases.get(predecessor).copied().unwrap_or_default();
            }
            if phase == 0 {
                continue;
            }
            for &id in &block.instructions {
                instruction_phases.insert(id, phase);
                if matches!(func.instructions[id.index()].value, InstructionValue::Await { .. }) {
                    phase = DEFERRED;
                }
            }
            if phases.get(&id) != Some(&phase) {
                phases.insert(id, phase);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let mut origins: Captures =
        contexts.iter().map(|&id| (id, FxHashSet::from_iter([id]))).collect();
    propagate_capture_origins(func, &mut origins, &FxHashSet::default());
    let mut before = Captures::default();
    let mut after = Captures::default();
    for (&id, &phase) in &instruction_phases {
        let instruction = &func.instructions[id.index()];
        if let InstructionValue::LoadContext { place, .. }
        | InstructionValue::LoadLocal { place, .. } = &instruction.value
            && contexts.contains(&place.identifier)
        {
            if phase & EAGER != 0 {
                before.entry(instruction.lvalue.identifier).or_default().insert(place.identifier);
            }
            if phase & DEFERRED != 0 {
                after.entry(instruction.lvalue.identifier).or_default().insert(place.identifier);
            }
        }
    }
    propagate_capture_origins(func, &mut before, &contexts);
    propagate_capture_origins(func, &mut after, &contexts);
    for (&id, &phase) in &instruction_phases {
        for invocation in invocations.get(&id).into_iter().flatten() {
            if phase & EAGER != 0 {
                eager.extend(invocation.eager_captures.intersection(&contexts).copied());
                snapshots.extend(invocation.snapshot_captures.intersection(&contexts).copied());
            }
            if phase & DEFERRED != 0 {
                deferred.extend(invocation.eager_captures.intersection(&contexts).copied());
                deferred.extend(invocation.snapshot_captures.intersection(&contexts).copied());
            }
            deferred.extend(invocation.deferred_captures.intersection(&contexts).copied());
        }
        match &func.instructions[id.index()].value {
            InstructionValue::LoadContext { place, .. }
            | InstructionValue::LoadLocal { place, .. }
                if contexts.contains(&place.identifier)
                    && !member_reads.contains_key(&place.identifier) =>
            {
                if phase & EAGER != 0 {
                    eager.insert(place.identifier);
                }
                if phase & DEFERRED != 0 {
                    deferred.insert(place.identifier);
                }
            }
            InstructionValue::PropertyLoad { object, .. }
            | InstructionValue::ComputedLoad { object, .. }
            | InstructionValue::Destructure { value: object, .. } => {
                for &binding in origins.get(&object.identifier).into_iter().flatten() {
                    if phase & EAGER != 0 {
                        eager.insert(binding);
                    }
                    if phase & DEFERRED != 0 {
                        let saved = before
                            .get(&object.identifier)
                            .is_some_and(|origins| origins.contains(&binding));
                        if saved {
                            snapshots.insert(binding);
                        }
                        if !saved
                            || after
                                .get(&object.identifier)
                                .is_some_and(|origins| origins.contains(&binding))
                        {
                            deferred.insert(binding);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    (eager, deferred, snapshots)
}

/// Refine captures used only through named members. A direct escape, unknown
/// property, or nested closure keeps the whole capture. This lets a returned
/// closure select individual rest arguments without taint from unused siblings.
fn infer_capture_members(
    func: &HirFunction,
    env: &Environment<'_>,
    rest_arguments: bool,
) -> FxHashMap<IdentifierId, FxHashSet<String>> {
    let mut members: FxHashMap<_, FxHashSet<String>> = func
        .context
        .iter()
        .map(|place| place.identifier)
        .chain(parameter_bindings(func, env))
        .map(|id| (id, FxHashSet::default()))
        .collect();
    if members.is_empty() {
        return members;
    }
    let mut origins: Captures =
        members.keys().map(|&id| (id, FxHashSet::from_iter([id]))).collect();
    let mut keys = FxHashMap::default();
    let mut receivers = FxHashMap::default();
    for instruction in &func.instructions {
        match &instruction.value {
            InstructionValue::Primitive { value: PrimitiveValue::String(value), .. } => {
                if let Some(value) = value.as_str() {
                    keys.insert(instruction.lvalue.identifier, value.to_string());
                }
            }
            InstructionValue::Primitive { value: PrimitiveValue::Number(value), .. } => {
                keys.insert(instruction.lvalue.identifier, value.to_string());
            }
            InstructionValue::PropertyLoad { object, property, .. } => {
                if !rest_arguments || array_index(&property.to_string()).is_some() {
                    receivers.insert(instruction.lvalue.identifier, object.identifier);
                }
            }
            InstructionValue::ComputedLoad { object, property, .. }
                if !rest_arguments
                    || keys
                        .get(&property.identifier)
                        .is_some_and(|key| array_index(key).is_some()) =>
            {
                receivers.insert(instruction.lvalue.identifier, object.identifier);
            }
            _ => {}
        }
    }
    loop {
        let mut changed = false;
        let mut copy = |from, into| {
            let values = origins.get(&from).cloned().unwrap_or_default();
            let target = origins.entry(into).or_default();
            let old_len = target.len();
            target.extend(values);
            changed |= old_len != target.len();
        };
        for (_, block) in &func.body.blocks {
            for phi in &block.phis {
                for operand in phi.operands.values() {
                    copy(operand.identifier, phi.place.identifier);
                }
            }
            for &id in &block.instructions {
                let instruction = &func.instructions[id.index()];
                match &instruction.value {
                    InstructionValue::LoadLocal { place, .. }
                    | InstructionValue::LoadContext { place, .. } => {
                        copy(place.identifier, instruction.lvalue.identifier)
                    }
                    InstructionValue::TypeCastExpression { value, .. } => {
                        copy(value.identifier, instruction.lvalue.identifier)
                    }
                    InstructionValue::StoreLocal { lvalue, value, .. }
                    | InstructionValue::StoreContext { lvalue, value, .. } => {
                        copy(value.identifier, lvalue.place.identifier);
                        copy(value.identifier, instruction.lvalue.identifier);
                    }
                    _ => {}
                }
            }
        }
        if !changed {
            break;
        }
    }
    let remove = |id, members: &mut FxHashMap<IdentifierId, FxHashSet<String>>| {
        for origin in origins.get(&id).into_iter().flatten() {
            members.remove(origin);
        }
    };
    for instruction in &func.instructions {
        match &instruction.value {
            InstructionValue::PropertyLoad { object, property, .. } => {
                for origin in origins.get(&object.identifier).into_iter().flatten() {
                    if let Some(members) = members.get_mut(origin) {
                        members.insert(property.to_string());
                    }
                }
            }
            InstructionValue::ComputedLoad { object, property, .. } => {
                if let Some(key) = keys.get(&property.identifier) {
                    for origin in origins.get(&object.identifier).into_iter().flatten() {
                        if let Some(members) = members.get_mut(origin) {
                            members.insert(key.clone());
                        }
                    }
                } else {
                    remove(object.identifier, &mut members);
                }
            }
            InstructionValue::LoadLocal { .. }
            | InstructionValue::LoadContext { .. }
            | InstructionValue::StoreLocal { .. }
            | InstructionValue::StoreContext { .. }
            | InstructionValue::TypeCastExpression { .. } => {}
            InstructionValue::FunctionExpression { lowered_func, .. }
            | InstructionValue::ObjectMethod { lowered_func, .. } => {
                for context in &env.functions[lowered_func.func].context {
                    remove(context.identifier, &mut members);
                }
            }
            value => {
                for operand in each_instruction_value_operand(value, env) {
                    if let InstructionValue::MethodCall { receiver, property, .. } = value
                        && operand.identifier == receiver.identifier
                        && receivers.get(&property.identifier) == Some(&receiver.identifier)
                    {
                        continue;
                    }
                    remove(operand.identifier, &mut members);
                }
            }
        }
    }
    for (_, block) in &func.body.blocks {
        for operand in each_terminal_operand(&block.terminal) {
            remove(operand.identifier, &mut members);
        }
    }
    members
}

// Named members include every reaching write to that member. The unknown-key
// bucket is only the fallback for names that have not been written explicitly.
/// The array index a property key names: a canonical decimal below
/// 2^32 - 1. Other numeric-looking keys such as "01" or "4294967295" are
/// ordinary named properties.
fn array_index(key: &str) -> Option<usize> {
    let index: u32 = key.parse().ok()?;
    (index != u32::MAX && index.to_string() == key).then(|| usize::try_from(index).ok())?
}

fn merge_properties(into: &mut Properties, other: &Properties) {
    merge_properties_in(into, other, None);
}

/// Joins two views of one object. A key present on one side only takes the
/// other side's unknown-key writes that could have named it. For an array,
/// `named` holds each side's generic unknown-key writes (`into`, `other`): a
/// named key falls back to those, never to the index summary, which index-only
/// writes such as `push` also feed.
fn merge_properties_in(
    into: &mut Properties,
    other: &Properties,
    named: Option<(&Values, &Values)>,
) {
    let fallback =
        |properties: &Properties, named: Option<&Values>, key: &Option<String>| match named {
            Some(named) if key.as_ref().is_some_and(|key| array_index(key).is_none()) => {
                Some(named.clone())
            }
            _ => properties.get(&None).cloned(),
        };
    let into_snapshot = into.clone();
    for (key, values) in into.iter_mut() {
        if let Some(other) = other.get(key) {
            values.extend(other);
        } else if let Some(other) = fallback(other, named.map(|(_, other)| other), key) {
            values.extend(&other);
        }
    }
    for (key, values) in other {
        if !into.contains_key(key) {
            let mut values = values.clone();
            if let Some(own) = fallback(&into_snapshot, named.map(|(into, _)| into), key) {
                values.extend(&own);
            }
            into.insert(key.clone(), values);
        }
    }
}

pub(super) struct AsyncInvocation {
    pub reassignment: Option<(IdentifierId, Option<Span>)>,
    pub deferred_captures: FxHashSet<IdentifierId>,
    pub snapshot_captures: FxHashSet<IdentifierId>,
    pub eager_captures: FxHashSet<IdentifierId>,
    pub context_values: Rc<Captures>,
}

pub(super) struct AsyncCallables {
    pub identifiers: FxHashSet<IdentifierId>,
    pub invocations: FxHashMap<InstructionId, Vec<AsyncInvocation>>,
    pub returning_callbacks: FxHashMap<InstructionId, FxHashSet<IdentifierId>>,
    pub returned: ReturnedCallables,
    pub pending_before: FxHashMap<InstructionId, AsyncInvocation>,
    pub pending_at_exit: FxHashMap<BlockId, AsyncInvocation>,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct ReturnedCallables {
    calls: FxHashMap<InstructionId, Vec<Call>>,
    pub generator: bool,
    pub generator_reassignment: Option<(IdentifierId, Option<Span>)>,
    values: Values,
    external_contexts: FxHashSet<IdentifierId>,
    params: Vec<(IdentifierId, bool)>,
    capture_members: FxHashMap<IdentifierId, FxHashSet<String>>,
    capture_reads: FxHashMap<IdentifierId, FxHashSet<String>>,
    eager_captures: FxHashSet<IdentifierId>,
    deferred_captures: FxHashSet<IdentifierId>,
    snapshot_captures: FxHashSet<IdentifierId>,
    projections: Projections,
    deferred_calls: DeferredCalls,
    parameter_values: Captures,
}

#[derive(Clone, PartialEq, Eq)]
struct Callable {
    is_async: bool,
    generator: bool,
    deferred_captures: FxHashSet<IdentifierId>,
    snapshot_captures: FxHashSet<IdentifierId>,
    context_values: Rc<Captures>,
    returned: Option<Rc<ReturnedCallables>>,
    capture_members: FxHashMap<IdentifierId, FxHashSet<String>>,
    capture_reads: FxHashMap<IdentifierId, FxHashSet<String>>,
    eager_captures: FxHashSet<IdentifierId>,
    bound_arguments: Arguments,
}

/// Pair each possible function with the live bindings on the paths where
/// that function is selected. Merging another callable must not mix its bindings
/// into this function's captures.
#[derive(Clone, Default, PartialEq, Eq)]
struct Values {
    identities: FxHashSet<IdentifierId>,
    contexts: FxHashMap<IdentifierId, Callable>,
    iterators: FxHashMap<IdentifierId, Callable>,
}

impl Values {
    fn single(identifier: IdentifierId) -> Self {
        Self { identities: FxHashSet::from_iter([identifier]), ..Self::default() }
    }

    fn is_empty(&self) -> bool {
        self.identities.is_empty() && self.contexts.is_empty() && self.iterators.is_empty()
    }

    fn extend(&mut self, other: &Self) {
        self.identities.extend(&other.identities);
        for (target, source) in
            [(&mut self.contexts, &other.contexts), (&mut self.iterators, &other.iterators)]
        {
            for (&function, callable) in source {
                let Some(into) = target.get_mut(&function) else {
                    target.insert(function, callable.clone());
                    continue;
                };
                into.bound_arguments.merge(&callable.bound_arguments);
                if Rc::ptr_eq(&into.context_values, &callable.context_values) {
                    continue;
                }
                let into = Rc::make_mut(&mut into.context_values);
                for (&binding, values) in callable.context_values.iter() {
                    into.entry(binding).or_default().extend(values);
                }
            }
        }
    }

    fn update_context(&mut self, binding: IdentifierId, values: &FxHashSet<IdentifierId>) {
        for callable in self.contexts.values_mut().chain(self.iterators.values_mut()) {
            Rc::make_mut(&mut callable.context_values).insert(binding, values.clone());
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
struct PendingCapture {
    roots: FxHashSet<IdentifierId>,
    snapshots: FxHashSet<IdentifierId>,
    live: bool,
    members: Option<FxHashSet<String>>,
    observed: FxHashSet<IdentifierId>,
    dependencies: FxHashMap<IdentifierId, Option<FxHashSet<String>>>,
}

fn merge_members(into: &mut Option<FxHashSet<String>>, other: &Option<FxHashSet<String>>) {
    if let Some(into_members) = into {
        if let Some(other) = other {
            into_members.extend(other.iter().cloned());
        } else {
            *into = None;
        }
    }
}

impl PendingCapture {
    fn merge(&mut self, other: &Self) {
        self.roots.extend(&other.roots);
        self.snapshots.extend(&other.snapshots);
        self.live |= other.live;
        merge_members(&mut self.members, &other.members);
        self.observed.extend(&other.observed);
        for (&id, members) in &other.dependencies {
            if let Some(into) = self.dependencies.get_mut(&id) {
                merge_members(into, members);
            } else {
                self.dependencies.insert(id, members.clone());
            }
        }
    }
}

/// `keys` and `values` hold every key and value that may be present. A Map also
/// splits its values by primitive key so that a definite `set` replaces and a
/// definite `delete` removes the earlier value; unknown keys stay in `unkeyed`.
#[derive(Clone, Default, PartialEq, Eq)]
struct CollectionContents {
    keys: Values,
    values: Values,
    map: bool,
    keyed: FxHashMap<PrimitiveKey, Values>,
    unkeyed: Values,
}

impl CollectionContents {
    fn extend(&mut self, other: &Self) {
        self.keys.extend(&other.keys);
        self.values.extend(&other.values);
        self.map |= other.map;
        for (key, values) in &other.keyed {
            self.keyed.entry(key.clone()).or_default().extend(values);
        }
        self.unkeyed.extend(&other.unkeyed);
    }

    fn rebuild_map_values(&mut self) {
        let mut values = self.unkeyed.clone();
        for keyed in self.keyed.values() {
            values.extend(keyed);
        }
        self.values = values;
    }
}

fn remove_identity(values: &mut Values, identity: IdentifierId) {
    values.identities.remove(&identity);
    values.contexts.remove(&identity);
    values.iterators.remove(&identity);
}

/// A primitive's type and its property-key spelling. Property access only
/// uses `text`, while Map keys also compare `kind`, so `1` and `"1"` differ.
#[derive(Clone, PartialEq, Eq, Hash)]
struct PrimitiveKey {
    kind: PrimitiveKind,
    text: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum PrimitiveKind {
    String,
    Number,
    BigInt,
    Boolean,
    Null,
    Undefined,
}

/// Mints one synthetic pair identity per Map and mutation site, numbered after
/// the function's real identifiers. Every block state shares the table, so a
/// site always yields the same pair across fixpoint iterations.
#[derive(Clone, Default)]
struct EntryIds(Rc<RefCell<EntryTable>>);

#[derive(Default)]
struct EntryTable {
    first: usize,
    next: usize,
    ids: FxHashMap<(IdentifierId, IdentifierId), IdentifierId>,
}

impl EntryIds {
    fn new(first: usize) -> Self {
        Self(Rc::new(RefCell::new(EntryTable { first, next: first, ids: FxHashMap::default() })))
    }

    fn contains(&self, id: IdentifierId) -> bool {
        let table = self.0.borrow();
        table.next > table.first && id.index() >= table.first
    }

    fn get(&self, map: IdentifierId, site: IdentifierId) -> IdentifierId {
        let mut table = self.0.borrow_mut();
        if let Some(&id) = table.ids.get(&(map, site)) {
            return id;
        }
        let id = IdentifierId::from_usize(table.next);
        table.next += 1;
        table.ids.insert((map, site), id);
        id
    }
}

impl PartialEq for EntryIds {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for EntryIds {}

/// Whether a method call may run a program-stored own property rather than
/// the built-in method of that name.
enum MethodOverride {
    None,
    Possible,
    Definite,
}

/// How an array mutation moves its index slots.
#[derive(Clone, Copy)]
enum Reorder<'a> {
    Reverse,
    Shift,
    Pop,
    /// Inserted at the front, in order.
    Unshift(&'a Arguments),
    /// Moves slots in a way the model does not track, optionally inserting
    /// values at unknown positions.
    Scramble(Option<&'a Values>),
}

#[derive(Clone, Default, PartialEq, Eq)]
struct CallableState {
    values: FxHashMap<IdentifierId, Values>,
    context_values: Captures,
    properties: FxHashMap<IdentifierId, Properties>,
    keys: FxHashMap<IdentifierId, PrimitiveKey>,
    builtin_function_methods: FxHashMap<IdentifierId, String>,
    array_lengths: FxHashMap<IdentifierId, usize>,
    arrays: FxHashSet<IdentifierId>,
    // Values written into an array through an unknown key, which may name a
    // non-index property. Index-only writes such as `push` stay out of it, so
    // named reads see only writes that could have targeted them.
    named_unknown: FxHashMap<IdentifierId, Values>,
    // Object literals without spread properties, so every key they list is
    // certainly present.
    exact_objects: FxHashSet<IdentifierId>,
    /// Own named properties every path has definitely stored and not deleted.
    present: FxHashMap<IdentifierId, FxHashSet<String>>,
    collections: FxHashMap<IdentifierId, CollectionContents>,
    // The synthetic `[key, value]` entry objects that may stand for each Map.
    // A pair is a snapshot: later Map mutations create a new pair instead of
    // changing one that may already be stored elsewhere.
    map_entries: FxHashMap<IdentifierId, FxHashSet<IdentifierId>>,
    entry_ids: EntryIds,
    // Member names of property loads and names of loaded globals, used to
    // recognize built-in methods that have no canonical signature name.
    members: FxHashMap<IdentifierId, String>,
    globals: FxHashMap<IdentifierId, String>,
    projections: Projections,
    deferred_calls: DeferredCalls,
    pending: FxHashMap<IdentifierId, PendingCapture>,
}

impl CallableState {
    fn refresh_capture(&self, capture: &mut PendingCapture) {
        capture.observed.clear();
        capture.dependencies.clear();
        let mut pending: Vec<_> = capture
            .roots
            .iter()
            .chain(&capture.snapshots)
            .map(|&id| (id, capture.members.clone()))
            .collect();
        while let Some((id, mut members)) = pending.pop() {
            let mut collection_callback = false;
            if members.as_ref().is_some_and(|members| members.contains("forEach"))
                && !self.properties.get(&id).is_some_and(|properties| {
                    properties.contains_key(&Some("forEach".to_string()))
                        || properties.contains_key(&None)
                })
                && let Some(contents) = self.collections.get(&id)
            {
                collection_callback = true;
                for value in contents.keys.identities.iter().chain(&contents.values.identities) {
                    pending.push((*value, None));
                }
            }
            if self.arrays.contains(&id)
                && members.as_ref().is_some_and(|members| {
                    members.iter().any(|member| {
                        matches!(
                            member.as_str(),
                            "forEach"
                                | "map"
                                | "flatMap"
                                | "filter"
                                | "find"
                                | "findIndex"
                                | "findLast"
                                | "findLastIndex"
                                | "some"
                                | "every"
                                | "reduce"
                                | "reduceRight"
                                | "sort"
                                | "toSorted"
                        ) && self.properties.get(&id).is_some_and(|properties| {
                            !properties.contains_key(&Some(member.clone()))
                        })
                    })
                })
            {
                // Inherited array callback consumers can pass any element to
                // their callback. An own method still selects only that field.
                members = None;
            }
            if let Some(existing) = capture.dependencies.get_mut(&id) {
                let old = existing.clone();
                merge_members(existing, &members);
                if *existing == old {
                    continue;
                }
            } else {
                capture.dependencies.insert(id, members.clone());
            }
            let callable = self.values.get(&id).filter(|values| !values.contexts.is_empty());
            if members.is_none()
                && let Some(callable) = callable
            {
                capture.observed.insert(id);
                capture.observed.extend(callable.contexts.keys().copied());
                continue;
            }
            let Some(properties) = self.properties.get(&id) else {
                if !collection_callback {
                    capture.observed.insert(id);
                }
                continue;
            };
            let mut add = |values: &Values| {
                pending.extend(values.identities.iter().map(|&id| (id, None)));
            };
            if let Some(members) = &members {
                for member in members {
                    if let Some(values) =
                        properties.get(&Some(member.clone())).or_else(|| properties.get(&None))
                    {
                        add(values);
                    }
                }
            } else {
                for values in properties.values() {
                    add(values);
                }
            }
        }
    }

    fn capture(&self, binding: IdentifierId, callable: &Callable) -> PendingCapture {
        let mut capture = PendingCapture {
            roots: callable
                .context_values
                .get(&binding)
                .cloned()
                .unwrap_or_else(|| FxHashSet::from_iter([binding])),
            members: callable.capture_reads.get(&binding).cloned(),
            ..PendingCapture::default()
        };
        self.refresh_capture(&mut capture);
        capture
    }

    fn invocation(
        &mut self,
        mut callable: Callable,
        arguments: Option<&Arguments>,
    ) -> AsyncInvocation {
        if let Some(arguments) = arguments
            && let Some(summary) = &callable.returned
        {
            let (substitutions, rest) = self.call_substitutions(&callable, summary, arguments);
            for (&binding, sources) in &summary.parameter_values {
                let mut flattened_rest = false;
                let identities = sources
                    .iter()
                    .flat_map(|&source| {
                        if let Some(elements) = rest.get(&source) {
                            flattened_rest = true;
                            let members = summary.capture_reads.get(&binding);
                            return elements
                                .iter()
                                .filter(|(key, _)| {
                                    key.as_ref().is_none_or(|key| {
                                        members.is_none_or(|members| members.contains(key))
                                    })
                                })
                                .flat_map(|(_, values)| values.identities.iter().copied())
                                .collect();
                        }
                        self.substitute(
                            source,
                            &substitutions,
                            &summary.projections,
                            &summary.deferred_calls,
                            &rest,
                            &mut FxHashSet::default(),
                        )
                        .map_or_else(|| FxHashSet::from_iter([source]), |values| values.identities)
                    })
                    .collect();
                if flattened_rest {
                    callable.capture_reads.remove(&binding);
                }
                Rc::make_mut(&mut callable.context_values).insert(binding, identities);
            }
            for &(parameter, _) in &summary.params {
                if !summary.parameter_values.contains_key(&parameter)
                    && let Some(values) = substitutions.get(&parameter)
                {
                    Rc::make_mut(&mut callable.context_values)
                        .insert(parameter, values.identities.clone());
                }
                if let Some(elements) = rest.get(&parameter)
                    && let Some(members) = summary.capture_members.get(&parameter)
                {
                    let identities = members
                        .iter()
                        .filter(|member| array_index(member).is_some())
                        .flat_map(|member| {
                            elements
                                .get(&Some(member.clone()))
                                .into_iter()
                                .chain(elements.get(&None))
                        })
                        .flat_map(|values| values.identities.iter().copied())
                        .collect();
                    Rc::make_mut(&mut callable.context_values).insert(parameter, identities);
                }
            }
        }
        let mut contexts = (*callable.context_values).clone();
        for &binding in &callable.eager_captures {
            contexts.insert(binding, self.capture(binding, &callable).observed);
        }
        for &binding in callable.deferred_captures.union(&callable.snapshot_captures) {
            let mut capture = self.capture(binding, &callable);
            capture.live = callable.deferred_captures.contains(&binding);
            if callable.snapshot_captures.contains(&binding) {
                capture.snapshots.clone_from(&capture.roots);
                if !capture.live {
                    capture.roots.clear();
                }
            }
            if let Some(current) = self.pending.get_mut(&binding) {
                current.merge(&capture);
            } else {
                self.pending.insert(binding, capture);
            }
        }
        AsyncInvocation {
            reassignment: callable
                .returned
                .as_ref()
                .and_then(|summary| summary.generator_reassignment),
            eager_captures: callable.eager_captures,
            deferred_captures: callable.deferred_captures,
            snapshot_captures: callable.snapshot_captures,
            context_values: Rc::new(contexts),
        }
    }

    fn execute_call(
        &mut self,
        call: &Call,
        visiting: &mut FxHashSet<IdentifierId>,
        invocations: &mut Vec<AsyncInvocation>,
    ) {
        for (&id, callable) in &call.callees.contexts {
            if callable.generator || !visiting.insert(id) {
                continue;
            }
            if callable.is_async {
                invocations.push(self.invocation(callable.clone(), call.arguments.as_ref()));
            } else if let Some(summary) = &callable.returned {
                let empty = Arguments::default();
                let (substitutions, rest) = self.call_substitutions(
                    callable,
                    summary,
                    call.arguments.as_ref().unwrap_or(&empty),
                );
                for nested in summary.calls.values().flatten() {
                    let callees = self.substitute_values(
                        &nested.callees,
                        summary,
                        &substitutions,
                        &rest,
                        &mut FxHashSet::default(),
                    );
                    let arguments = nested.arguments.as_ref().map(|arguments| Arguments {
                        length: arguments.length,
                        elements: arguments
                            .elements
                            .iter()
                            .map(|(key, values)| {
                                (
                                    key.clone(),
                                    self.substitute_values(
                                        values,
                                        summary,
                                        &substitutions,
                                        &rest,
                                        &mut FxHashSet::default(),
                                    ),
                                )
                            })
                            .collect(),
                    });
                    self.execute_call(&Call { callees, arguments }, visiting, invocations);
                }
            }
            visiting.remove(&id);
        }
    }

    fn pending_observation(&self) -> AsyncInvocation {
        let mut contexts = self.context_values.clone();
        for (&binding, capture) in &self.pending {
            contexts.insert(binding, capture.observed.clone());
        }
        AsyncInvocation {
            reassignment: None,
            eager_captures: FxHashSet::default(),
            deferred_captures: self.pending.keys().copied().collect(),
            snapshot_captures: FxHashSet::default(),
            context_values: Rc::new(contexts),
        }
    }

    fn invoked_callbacks(
        &self,
        instruction: &InstructionValue<'_>,
        env: &Environment<'_>,
        require_result_flow: bool,
    ) -> Values {
        let args = match instruction {
            InstructionValue::CallExpression { args, .. }
            | InstructionValue::MethodCall { args, .. } => args,
            _ => return Values::default(),
        };
        let arguments = self.arguments(args);
        let count = arguments.length.unwrap_or_else(|| {
            arguments
                .elements
                .keys()
                .filter_map(|key| array_index(key.as_ref()?))
                .max()
                .unwrap_or(0)
                + 1
        });
        let mut result = Values::default();
        for index in no_alias_callback_parameters(instruction, env, require_result_flow, count) {
            if let Some(values) = arguments.elements.get(&Some(index.to_string())) {
                result.extend(values);
            }
            if let Some(values) = arguments.elements.get(&None) {
                result.extend(values);
            }
        }
        result
    }

    fn callback_arguments(
        &self,
        instruction: &InstructionValue<'_>,
        env: &Environment<'_>,
    ) -> Option<Arguments> {
        let InstructionValue::MethodCall { receiver, property, .. } = instruction else {
            return None;
        };
        let ty = &env.types[env.identifiers[property.identifier].type_];
        let signature = env.get_function_signature(ty).ok().flatten()?;
        if !matches!(
            signature.canonical_name.as_deref(),
            Some(
                "Array.map"
                    | "Array.flatMap"
                    | "Array.forEach"
                    | "Array.filter"
                    | "Array.every"
                    | "Array.some"
                    | "Array.find"
                    | "Array.findIndex"
                    | "Set.forEach"
                    | "Map.forEach"
            )
        ) {
            return None;
        }
        let mut items = Values::default();
        let mut keys = Values::default();
        let collection_callback =
            matches!(signature.canonical_name.as_deref(), Some("Set.forEach" | "Map.forEach"));
        if collection_callback {
            for id in self.values(receiver.identifier).identities {
                if let Some(contents) = self.collections.get(&id) {
                    items.extend(&contents.values);
                    keys.extend(&contents.keys);
                }
            }
        }
        for (key, values) in
            self.properties(receiver.identifier).into_iter().filter(|_| !collection_callback)
        {
            if key.as_ref().is_none_or(|key| array_index(key).is_some()) {
                items.extend(&values);
            }
        }
        Some(Arguments {
            elements: FxHashMap::from_iter([
                (Some("0".to_string()), items),
                (Some("1".to_string()), keys),
                (Some("2".to_string()), self.values(receiver.identifier)),
            ]),
            length: Some(3),
        })
    }

    fn update_collection(&mut self, instruction: &Instruction<'_>, env: &Environment<'_>) {
        let (callee, args) = match &instruction.value {
            InstructionValue::NewExpression { callee, args, .. } => (callee.identifier, args),
            InstructionValue::MethodCall { property, args, .. } => (property.identifier, args),
            _ => return,
        };
        let ty = &env.types[env.identifiers[callee].type_];
        let Some(signature) = env.get_function_signature(ty).ok().flatten() else { return };
        let arguments = self.arguments(args);
        let argument = |index: usize| arguments.at(index);
        match (&instruction.value, signature.canonical_name.as_deref()) {
            (InstructionValue::NewExpression { .. }, Some(name @ ("Set" | "Map"))) => {
                let map = name == "Map";
                let mut contents = CollectionContents { map, ..CollectionContents::default() };
                // Iterating the source yields Set values, Map entry pairs, or
                // array elements. A Map reads key and value from each pair.
                let mut items = Values::default();
                for source in argument(0).identities {
                    if let Some(existing) = self.collections.get(&source) {
                        if existing.map == map {
                            contents.extend(existing);
                            continue;
                        }
                        if existing.map {
                            for &entry in self.map_entries.get(&source).into_iter().flatten() {
                                items.extend(&Values::single(entry));
                            }
                        } else {
                            items.extend(&existing.values);
                        }
                    } else if let Some(properties) = self.properties.get(&source) {
                        for (key, item) in properties {
                            if key.as_ref().is_none_or(|key| array_index(key).is_some()) {
                                items.extend(item);
                            }
                        }
                    }
                }
                if map {
                    contents.keys.extend(&self.project(&items, &Some("0".to_string())));
                    contents.unkeyed.extend(&self.project(&items, &Some("1".to_string())));
                    contents.rebuild_map_values();
                } else {
                    contents.keys.extend(&items);
                    contents.values.extend(&items);
                }
                let id = instruction.lvalue.identifier;
                self.allocate(id);
                self.collections.insert(id, contents);
                if map {
                    self.sync_map_entries(id, id);
                }
            }
            (
                InstructionValue::MethodCall { receiver, .. },
                Some(
                    "Set.add" | "Map.set" | "Set.clear" | "Map.clear" | "Set.delete" | "Map.delete",
                ),
            ) => {
                // Contents are updated by `update_collection_method`. Typed
                // receivers that are not tracked still return themselves.
                let objects = self.values(receiver.identifier);
                self.refresh_pending(&objects, Some(&"forEach".to_string()));
                if matches!(signature.canonical_name.as_deref(), Some("Set.add" | "Map.set")) {
                    self.values.insert(instruction.lvalue.identifier, objects);
                }
            }
            _ => {}
        }
    }

    fn update_builtins(&mut self, instruction: &Instruction<'_>, env: &Environment<'_>) {
        self.update_collection_method(instruction);
        self.update_collection(instruction, env);
        self.update_transfer(instruction);
    }

    /// Whether a method call may run an own property the program stored over
    /// the built-in method of the same name. Built-in methods load as empty
    /// values, so a tracked callee means a stored function may run instead.
    fn method_override(&self, instruction: &Instruction<'_>) -> MethodOverride {
        let InstructionValue::MethodCall { receiver, property, .. } = &instruction.value else {
            return MethodOverride::None;
        };
        if self.values(property.identifier).is_empty() {
            return MethodOverride::None;
        }
        let Some(name) = self.members.get(&property.identifier) else {
            return MethodOverride::Possible;
        };
        let objects = self.values(receiver.identifier).identities;
        if !objects.is_empty()
            && objects
                .iter()
                .all(|object| self.present.get(object).is_some_and(|keys| keys.contains(name)))
        {
            MethodOverride::Definite
        } else {
            MethodOverride::Possible
        }
    }

    /// Collection methods are matched by member name because `delete`, `get`
    /// and friends carry no canonical signature name.
    fn update_collection_method(&mut self, instruction: &Instruction<'_>) {
        let InstructionValue::MethodCall { receiver, property, args, .. } = &instruction.value
        else {
            return;
        };
        let Some(name) = self.members.get(&property.identifier).cloned() else { return };
        let objects = self.values(receiver.identifier);
        if !objects.identities.iter().any(|object| self.collections.contains_key(object)) {
            return;
        }
        let arguments = self.arguments(args);
        let key = arguments.at(0);
        let value = arguments.at(1);
        let key_name = self.argument_primitive(args, 0).cloned();
        let definite = self.singleton(&objects);
        let mut result = None;
        for object in &objects.identities {
            let removable = self.removable_identity(*object, &key);
            let Some(contents) = self.collections.get_mut(object) else { continue };
            match (name.as_str(), contents.map) {
                ("add", false) => {
                    contents.keys.extend(&key);
                    contents.values.extend(&key);
                }
                ("set", true) => {
                    contents.keys.extend(&key);
                    match &key_name {
                        Some(key_name) if definite => {
                            contents.keyed.insert(key_name.clone(), value.clone());
                        }
                        Some(key_name) => {
                            contents.keyed.entry(key_name.clone()).or_default().extend(&value);
                        }
                        None => contents.unkeyed.extend(&value),
                    }
                    contents.rebuild_map_values();
                }
                ("clear", map) if definite => {
                    *contents = CollectionContents { map, ..CollectionContents::default() };
                }
                ("delete", true) if definite => {
                    if let Some(key_name) = &key_name {
                        contents.keyed.remove(key_name);
                        contents.rebuild_map_values();
                    }
                }
                ("delete", false) if definite => {
                    if let Some(identity) = removable {
                        remove_identity(&mut contents.keys, identity);
                        remove_identity(&mut contents.values, identity);
                    }
                }
                ("get", true) => {
                    let values = result.get_or_insert_with(Values::default);
                    match key_name.as_ref().and_then(|key_name| contents.keyed.get(key_name)) {
                        Some(keyed) => {
                            values.extend(keyed);
                            values.extend(&contents.unkeyed);
                        }
                        None if key_name.is_some() => values.extend(&contents.unkeyed),
                        None => values.extend(&contents.values),
                    }
                }
                _ => continue,
            }
            if contents.map && name != "get" {
                self.sync_map_entries(*object, instruction.lvalue.identifier);
            }
        }
        match name.as_str() {
            "get" => {
                if let Some(result) = result {
                    self.values.insert(instruction.lvalue.identifier, result);
                }
            }
            "add" | "set" => {
                self.values.insert(instruction.lvalue.identifier, objects.clone());
                self.refresh_pending(&objects, Some(&"forEach".to_string()));
            }
            "clear" | "delete" => self.refresh_pending(&objects, Some(&"forEach".to_string())),
            _ => {}
        }
    }

    /// `Set.delete(value)` removes `value` only when it is one function
    /// identity that no other member can also carry, such as a bound copy.
    fn removable_identity(&self, set: IdentifierId, key: &Values) -> Option<IdentifierId> {
        let contents = self.collections.get(&set)?;
        let mut identities = key.identities.iter();
        let identity = *identities.next()?;
        if identities.next().is_some() || !key.contexts.contains_key(&identity) {
            return None;
        }
        contents
            .values
            .identities
            .iter()
            .all(|&other| {
                other == identity
                    || contents.values.contexts.contains_key(&other)
                    || self.properties.contains_key(&other)
                    || self.collections.contains_key(&other)
            })
            .then_some(identity)
    }

    /// Points `map` at the `[key, value]` pair minted for `site`, holding the
    /// Map's current contents. Earlier pairs are left as is, because pairs
    /// already copied into another container or array keep the values they
    /// had. Only a loop that returns to `site` finds its pair already present;
    /// it merges, so a pair saved on an earlier iteration keeps its values.
    fn sync_map_entries(&mut self, map: IdentifierId, site: IdentifierId) {
        let Some(contents) = self.collections.get(&map) else { return };
        let entry = self.entry_ids.get(map, site);
        let properties = FxHashMap::from_iter([
            (Some("0".to_string()), contents.keys.clone()),
            (Some("1".to_string()), contents.values.clone()),
        ]);
        merge_properties(self.properties.entry(entry).or_default(), &properties);
        self.arrays.insert(entry);
        self.array_lengths.insert(entry, 2);
        self.map_entries.insert(map, FxHashSet::from_iter([entry]));
    }

    /// The property-key spelling of a known primitive.
    fn property_key(&self, id: IdentifierId) -> Option<String> {
        self.keys.get(&id).map(|key| key.text.clone())
    }

    /// The primitive at `index`, known only when no spread comes first.
    fn argument_primitive(&self, args: &[PlaceOrSpread], index: usize) -> Option<&PrimitiveKey> {
        let PlaceOrSpread::Place(place) = args.get(index)? else { return None };
        if args[..index].iter().any(|arg| matches!(arg, PlaceOrSpread::Spread(_))) {
            return None;
        }
        self.keys.get(&place.identifier)
    }

    /// The property-key spelling of the primitive at `index`.
    fn argument_key(&self, args: &[PlaceOrSpread], index: usize) -> Option<String> {
        self.argument_primitive(args, index).map(|key| key.text.clone())
    }

    /// Values produced by iterating `identifier`: array elements, Set values,
    /// or the `[key, value]` entries of a Map.
    fn elements(&self, identifier: IdentifierId) -> Values {
        self.elements_of(&self.values(identifier))
    }

    fn elements_of(&self, values: &Values) -> Values {
        let mut result = Values::default();
        for &object in &values.identities {
            if let Some(contents) = self.collections.get(&object) {
                if !contents.map {
                    result.extend(&contents.values);
                } else {
                    for &entry in self.map_entries.get(&object).into_iter().flatten() {
                        result.extend(&Values::single(entry));
                    }
                }
            } else if let Some(properties) = self.properties.get(&object) {
                for (key, values) in properties {
                    if key.as_ref().is_none_or(|key| array_index(key).is_some()) {
                        result.extend(values);
                    }
                }
            }
        }
        result
    }

    fn new_array(&mut self, id: IdentifierId, elements: Values) {
        self.allocate(id);
        self.arrays.insert(id);
        self.array_lengths.remove(&id);
        self.properties.insert(id, FxHashMap::from_iter([(None, elements)]));
    }

    /// Array and `Object` built-ins that move values between containers
    /// without invoking them. Anything else leaves its receiver unchanged.
    fn update_transfer(&mut self, instruction: &Instruction<'_>) {
        let InstructionValue::MethodCall { receiver, property, args, .. } = &instruction.value
        else {
            return;
        };
        let Some(name) = self.members.get(&property.identifier).cloned() else { return };
        let id = instruction.lvalue.identifier;
        let arguments = self.arguments(args);
        if let Some(global) = self.globals.get(&receiver.identifier).cloned() {
            match (global.as_str(), name.as_str()) {
                ("Array", "of") => self.new_array(id, arguments.from(0)),
                // A mapping callback is invoked; leave that call unmodeled.
                ("Array", "from") if arguments.length.is_none_or(|length| length == 1) => {
                    let elements = self.elements_of(&arguments.at(0));
                    self.new_array(id, elements);
                }
                ("Object", "assign") => {
                    // Sources are copied in argument order. A source overwrites
                    // only when it is certainly the object it names: the
                    // argument is that object's own allocation. Anything else
                    // (a conditional, a variable that may hold null, a spread
                    // source) may write nothing, so it merges weakly.
                    let target = arguments.at(0);
                    let mut positions: Vec<usize> = arguments
                        .elements
                        .keys()
                        .filter_map(|key| array_index(key.as_ref()?))
                        .filter(|&position| position >= 1)
                        .collect();
                    positions.sort_unstable();
                    for position in positions {
                        let sources = arguments.elements[&Some(position.to_string())].clone();
                        let weak = !self.fresh_object(args, position, &sources);
                        self.assign(&target, &sources, weak);
                    }
                    if let Some(sources) = arguments.elements.get(&None) {
                        self.assign(&target, sources, true);
                    }
                    self.values.insert(id, target);
                }
                _ => {}
            }
            return;
        }
        let receivers = self.values(receiver.identifier);
        if !receivers.identities.iter().any(|object| self.arrays.contains(object)) {
            return;
        }
        let elements = self.elements(receiver.identifier);
        match name.as_str() {
            "push" => self.store_index(&receivers, &arguments.from(0)),
            "fill" => {
                self.store_index(&receivers, &arguments.at(0));
                self.values.insert(id, receivers);
            }
            "unshift" => self.reorder(&receivers, Reorder::Unshift(&arguments)),
            "shift" => {
                self.values.insert(id, elements);
                self.reorder(&receivers, Reorder::Shift);
            }
            "pop" => {
                self.values.insert(id, elements);
                self.reorder(&receivers, Reorder::Pop);
            }
            "reverse" => {
                self.reorder(&receivers, Reorder::Reverse);
                self.values.insert(id, receivers);
            }
            "sort" | "copyWithin" => {
                self.reorder(&receivers, Reorder::Scramble(None));
                self.values.insert(id, receivers);
            }
            "splice" => {
                self.reorder(&receivers, Reorder::Scramble(Some(&arguments.from(2))));
                self.new_array(id, elements);
            }
            "flat" => {
                let depth = if args.is_empty() {
                    Some(1)
                } else {
                    self.argument_key(args, 0).and_then(|depth| depth.parse().ok())
                };
                let flattened = self.flatten(elements, depth);
                self.new_array(id, flattened);
            }
            "concat" | "toSpliced" | "with" => {
                let mut values = elements;
                let operands = arguments.from(usize::from(name != "concat"));
                values.extend(&operands);
                values.extend(&self.elements_of(&operands));
                self.new_array(id, values);
            }
            "slice" | "toReversed" | "toSorted" => self.new_array(id, elements),
            "at" => {
                self.values.insert(id, elements);
            }
            _ => {}
        }
    }

    /// Whether argument `position` is a plain place that is itself the one
    /// object allocation in `sources`, so it cannot be null or another value,
    /// and that allocation is an object literal whose keys are all present.
    fn fresh_object(&self, args: &[PlaceOrSpread], position: usize, sources: &Values) -> bool {
        if args[..position.min(args.len())]
            .iter()
            .any(|arg| matches!(arg, PlaceOrSpread::Spread(_)))
        {
            return false;
        }
        let Some(PlaceOrSpread::Place(place)) = args.get(position) else { return false };
        sources.identities.len() == 1
            && sources.identities.contains(&place.identifier)
            && self.exact_objects.contains(&place.identifier)
    }

    /// Copies the own properties of `sources` onto `target`. A weak copy keeps
    /// each existing value alongside the copied one.
    fn assign(&mut self, target: &Values, sources: &Values, weak: bool) {
        for source in &sources.identities {
            let properties = self.properties.get(source).cloned().unwrap_or_default();
            for (key, mut values) in properties {
                if weak && key.is_some() {
                    values.extend(&self.project(target, &key));
                }
                self.store_into(target, key, values);
            }
        }
    }

    /// Moves the index slots of the arrays in `objects`. An exact move needs
    /// one receiver of known length; otherwise every index slot joins the
    /// unknown-index bucket, so an indexed read sees any element that may
    /// have moved there.
    fn reorder(&mut self, objects: &Values, reorder: Reorder<'_>) {
        let definite = self.singleton(objects);
        for &object in &objects.identities {
            if !self.arrays.contains(&object) {
                continue;
            }
            let length = self.array_lengths.get(&object).copied().filter(|_| definite);
            let properties = self.properties.entry(object).or_default();
            let mut indexed = Vec::new();
            properties.retain(|key, values| match key.as_ref().and_then(|key| array_index(key)) {
                Some(index) => {
                    indexed.push((index, std::mem::take(values)));
                    false
                }
                _ => true,
            });
            let exact = length.and_then(|length| {
                let moved = |index: usize| match reorder {
                    Reorder::Reverse => length.checked_sub(index + 1),
                    Reorder::Shift => index.checked_sub(1),
                    Reorder::Pop => (index + 1 < length).then_some(index),
                    Reorder::Unshift(inserted) => Some(index + inserted.length?),
                    Reorder::Scramble(_) => None,
                };
                let new_length = match reorder {
                    Reorder::Reverse => length,
                    Reorder::Shift | Reorder::Pop => length.saturating_sub(1),
                    Reorder::Unshift(inserted) => {
                        if inserted.elements.contains_key(&None) {
                            return None;
                        }
                        length + inserted.length?
                    }
                    Reorder::Scramble(_) => return None,
                };
                let mut slots = Properties::default();
                for (index, values) in &indexed {
                    if *index >= length {
                        return None;
                    }
                    let removed = matches!(reorder, Reorder::Shift) && *index == 0
                        || matches!(reorder, Reorder::Pop) && *index + 1 == length;
                    if removed {
                        continue;
                    }
                    slots.insert(Some(moved(*index)?.to_string()), values.clone());
                }
                if let Reorder::Unshift(inserted) = reorder {
                    for (key, values) in &inserted.elements {
                        slots.insert(key.clone(), values.clone());
                    }
                }
                Some((slots, new_length))
            });
            if let Some((slots, new_length)) = exact {
                properties.extend(slots);
                self.array_lengths.insert(object, new_length);
            } else {
                let unknown = properties.entry(None).or_default();
                for (_, values) in &indexed {
                    unknown.extend(values);
                }
                match reorder {
                    Reorder::Unshift(inserted) => unknown.extend(&inserted.from(0)),
                    Reorder::Scramble(Some(inserted)) => unknown.extend(inserted),
                    _ => {}
                }
                self.array_lengths.remove(&object);
            }
        }
        self.refresh_pending(objects, None);
    }

    /// Elements of `elements` flattened `depth` levels, or every level when
    /// the depth is unknown. Nested arrays are kept alongside their elements.
    fn flatten(&self, mut elements: Values, depth: Option<usize>) -> Values {
        let mut seen = FxHashSet::default();
        let mut level = 0;
        while depth.is_none_or(|depth| level < depth) {
            let nested = Values {
                identities: elements
                    .identities
                    .iter()
                    .copied()
                    .filter(|id| self.arrays.contains(id) && seen.insert(*id))
                    .collect(),
                ..Values::default()
            };
            if nested.identities.is_empty() {
                break;
            }
            elements.extend(&self.elements_of(&nested));
            level += 1;
        }
        elements
    }

    fn values(&self, identifier: IdentifierId) -> Values {
        self.values.get(&identifier).cloned().unwrap_or_default()
    }

    fn copy(&mut self, from: IdentifierId, into: IdentifierId) {
        self.values.insert(into, self.values(from));
        if let Some(key) = self.keys.get(&from).cloned() {
            self.keys.insert(into, key);
        } else {
            self.keys.remove(&into);
        }
        if let Some(method) = self.builtin_function_methods.get(&from).cloned() {
            self.builtin_function_methods.insert(into, method);
        } else {
            self.builtin_function_methods.remove(&into);
        }
    }

    fn allocate(&mut self, identifier: IdentifierId) {
        self.values.insert(identifier, Values::single(identifier));
        self.named_unknown.remove(&identifier);
        self.exact_objects.remove(&identifier);
        self.present.remove(&identifier);
    }

    fn update_context(&mut self, binding: IdentifierId) {
        let values = self.values(binding).identities;
        if let Some(mut capture) = self.pending.remove(&binding) {
            if capture.live {
                capture.roots.clone_from(&values);
                self.refresh_capture(&mut capture);
            }
            self.pending.insert(binding, capture);
        }
        self.context_values.insert(binding, values.clone());
        for value in self.values.values_mut() {
            value.update_context(binding, &values);
        }
        for contents in self.collections.values_mut() {
            contents.keys.update_context(binding, &values);
            contents.values.update_context(binding, &values);
            contents.unkeyed.update_context(binding, &values);
            for keyed in contents.keyed.values_mut() {
                keyed.update_context(binding, &values);
            }
        }
        for properties in self.properties.values_mut() {
            for value in properties.values_mut() {
                value.update_context(binding, &values);
            }
        }
        for value in self.named_unknown.values_mut() {
            value.update_context(binding, &values);
        }
    }

    fn merge(&mut self, other: &Self) {
        for (&binding, capture) in &other.pending {
            if let Some(into) = self.pending.get_mut(&binding) {
                into.merge(capture);
            } else {
                self.pending.insert(binding, capture.clone());
            }
        }
        for (&binding, values) in &other.context_values {
            self.context_values.entry(binding).or_default().extend(values);
        }
        for (&id, values) in &other.values {
            self.values.entry(id).or_default().extend(values);
        }
        let empty = Values::default();
        for (&object, properties) in &other.properties {
            let named =
                (self.arrays.contains(&object) || other.arrays.contains(&object)).then(|| {
                    (
                        self.named_unknown.get(&object).cloned().unwrap_or_default(),
                        other.named_unknown.get(&object).unwrap_or(&empty),
                    )
                });
            merge_properties_in(
                self.properties.entry(object).or_default(),
                properties,
                named.as_ref().map(|(into, other)| (into, *other)),
            );
        }
        self.present.retain(|object, keys| {
            let Some(other) = other.present.get(object) else { return false };
            keys.retain(|key| other.contains(key));
            !keys.is_empty()
        });
        self.keys.retain(|id, key| other.keys.get(id) == Some(key));
        self.builtin_function_methods
            .retain(|id, method| other.builtin_function_methods.get(id) == Some(method));
        self.array_lengths.retain(|id, length| other.array_lengths.get(id) == Some(length));
        self.arrays.extend(&other.arrays);
        self.exact_objects.extend(&other.exact_objects);
        for (&object, values) in &other.named_unknown {
            self.named_unknown.entry(object).or_default().extend(values);
        }
        for (&id, contents) in &other.collections {
            self.collections.entry(id).or_default().extend(contents);
        }
        // Each path keeps its own pair, so a pair saved on one path never
        // gains the other path's values.
        for (&map, entries) in &other.map_entries {
            self.map_entries.entry(map).or_default().extend(entries);
        }
        self.members.retain(|id, name| other.members.get(id) == Some(name));
        self.globals.retain(|id, name| other.globals.get(id) == Some(name));
        for (&id, call) in &other.deferred_calls {
            if let Some(into) = self.deferred_calls.get_mut(&id) {
                into.merge(call);
            } else {
                self.deferred_calls.insert(id, call.clone());
            }
        }
        for (&id, projections) in &other.projections {
            self.projections.entry(id).or_default().extend(projections.iter().cloned());
        }
    }

    fn key(&self, property: &ObjectPropertyKey<'_>) -> Option<String> {
        match property {
            ObjectPropertyKey::Identifier { name, .. } | ObjectPropertyKey::String { name, .. } => {
                Some(name.to_string())
            }
            ObjectPropertyKey::Computed { name, .. } => self.property_key(name.identifier),
        }
    }

    fn properties(&self, identifier: IdentifierId) -> Properties {
        let mut result = Properties::default();
        for object in self.values.get(&identifier).into_iter().flat_map(|values| &values.identities)
        {
            if let Some(properties) = self.properties.get(object) {
                merge_properties(&mut result, properties);
            }
        }
        result
    }

    fn load(&mut self, object: IdentifierId, key: Option<String>, into: IdentifierId) {
        let properties = self.properties(object);
        if let Some(key) = &key
            && ((matches!(key.as_str(), "call" | "apply" | "bind")
                && !self.values(object).contexts.is_empty())
                || (key == "next" && !self.values(object).iterators.is_empty()))
            && !properties.contains_key(&Some(key.clone()))
            && !properties.contains_key(&None)
        {
            self.builtin_function_methods.insert(into, key.clone());
        } else {
            self.builtin_function_methods.remove(&into);
        }
        let mut values = Values::default();
        for object in self.values(object).identities {
            let Some(properties) = self.properties.get(&object) else { continue };
            if key.is_some() {
                if let Some(found) = self.lookup(object, properties, &key) {
                    values.extend(found);
                }
            } else {
                for found in properties.values() {
                    values.extend(found);
                }
            }
        }
        for object in self.values(object).identities {
            if self.projections.contains_key(&object) {
                self.projections
                    .entry(into)
                    .or_default()
                    .insert(Projection { object, key: key.clone() });
                values.identities.insert(into);
            }
        }
        self.values.insert(into, values);
        self.keys.remove(&into);
    }

    /// An unknown-key write to an array stands for some index, so it answers
    /// index reads but not named reads such as `push` or `length`.
    fn lookup<'p>(
        &'p self,
        object: IdentifierId,
        properties: &'p Properties,
        key: &Option<String>,
    ) -> Option<&'p Values> {
        properties.get(key).or_else(|| self.unknown_fallback(object, properties, key))
    }

    /// What an absent `key` may read through earlier unknown-key writes.
    fn unknown_fallback<'p>(
        &'p self,
        object: IdentifierId,
        properties: &'p Properties,
        key: &Option<String>,
    ) -> Option<&'p Values> {
        let named = key.as_ref().is_some_and(|key| array_index(key).is_none());
        if named && self.arrays.contains(&object) {
            self.named_unknown.get(&object)
        } else {
            properties.get(&None)
        }
    }

    fn invoked(&self, instruction: &InstructionValue<'_>) -> Values {
        match instruction {
            InstructionValue::CallExpression { callee, .. }
            | InstructionValue::NewExpression { callee, .. } => self.values(callee.identifier),
            InstructionValue::MethodCall { receiver, property, .. } => {
                if self
                    .builtin_function_methods
                    .get(&property.identifier)
                    .is_some_and(|method| method != "bind")
                {
                    let mut values = self.values(receiver.identifier);
                    if self.builtin_function_methods[&property.identifier] == "next" {
                        values.contexts = std::mem::take(&mut values.iterators);
                        for callable in values.contexts.values_mut() {
                            callable.generator = false;
                        }
                    }
                    values
                } else {
                    self.values(property.identifier)
                }
            }
            InstructionValue::TaggedTemplateExpression { tag, .. } => self.values(tag.identifier),
            _ => Values::default(),
        }
    }

    fn array_length(&self, values: &Values) -> Option<usize> {
        let mut lengths = values.identities.iter().map(|id| self.array_lengths.get(id).copied());
        let length = lengths.next()??;
        lengths.all(|other| other == Some(length)).then_some(length)
    }

    fn arguments(&self, args: &[PlaceOrSpread]) -> Arguments {
        let mut result = Arguments::default();
        for argument in args {
            let next = match argument {
                PlaceOrSpread::Place(place) => Arguments {
                    elements: FxHashMap::from_iter([(
                        Some("0".to_string()),
                        self.values(place.identifier),
                    )]),
                    length: Some(1),
                },
                PlaceOrSpread::Spread(spread) => self.spread(spread.place.identifier),
            };
            result.append(&next);
        }
        result
    }

    /// Spreading a Set or Map yields its values or entry pairs at unknown
    /// positions; spreading an array keeps its indexed elements.
    fn spread(&self, identifier: IdentifierId) -> Arguments {
        let mut arguments = Arguments {
            elements: self.properties(identifier),
            length: self.array_length(&self.values(identifier)),
        };
        if self.values(identifier).identities.iter().any(|id| self.collections.contains_key(id)) {
            arguments.elements.entry(None).or_default().extend(&self.elements(identifier));
            arguments.length = None;
        }
        arguments
    }

    fn project(&self, objects: &Values, key: &Option<String>) -> Values {
        let mut result = Values::default();
        for object in &objects.identities {
            let Some(properties) = self.properties.get(object) else { continue };
            if key.is_some() {
                if let Some(values) = self.lookup(*object, properties, key) {
                    result.extend(values);
                }
            } else {
                for values in properties.values() {
                    result.extend(values);
                }
            }
        }
        result
    }

    fn substitute(
        &self,
        id: IdentifierId,
        substitutions: &FxHashMap<IdentifierId, Values>,
        projections: &Projections,
        deferred_calls: &DeferredCalls,
        rest_arguments: &FxHashMap<IdentifierId, Properties>,
        visiting: &mut FxHashSet<IdentifierId>,
    ) -> Option<Values> {
        if let Some(values) = substitutions.get(&id) {
            return Some(values.clone());
        }
        if let Some(call) = deferred_calls.get(&id) {
            if !visiting.insert(id) {
                return None;
            }
            let mut callees = Values::default();
            for &callee in &call.callees {
                let selected = self
                    .substitute(
                        callee,
                        substitutions,
                        projections,
                        deferred_calls,
                        rest_arguments,
                        visiting,
                    )
                    .unwrap_or_else(|| self.values(callee));
                callees.extend(&selected);
            }
            let mut arguments = call.arguments.clone();
            for values in arguments.elements.values_mut() {
                for value in values.identities.clone() {
                    if let Some(selected) = self.substitute(
                        value,
                        substitutions,
                        projections,
                        deferred_calls,
                        rest_arguments,
                        visiting,
                    ) {
                        values.identities.remove(&value);
                        values.extend(&selected);
                    }
                }
            }
            let mut result = Values::default();
            for callable in callees.contexts.values() {
                result.extend(&self.returned_values(callable, &arguments, visiting));
            }
            if callees.contexts.is_empty() {
                result.identities.insert(id);
            }
            visiting.remove(&id);
            return Some(result);
        }
        let sources = projections.get(&id)?;
        if sources.is_empty() || !visiting.insert(id) {
            return None;
        }
        let mut result = Values::default();
        for source in sources {
            if let Some(elements) = rest_arguments.get(&source.object) {
                if source.key.is_some() {
                    if let Some(values) = elements.get(&source.key).or_else(|| elements.get(&None))
                    {
                        result.extend(values);
                    }
                } else {
                    for values in elements.values() {
                        result.extend(values);
                    }
                }
                continue;
            }
            if let Some(objects) = self.substitute(
                source.object,
                substitutions,
                projections,
                deferred_calls,
                rest_arguments,
                visiting,
            ) {
                result.extend(&self.project(&objects, &source.key));
            } else {
                result.identities.insert(id);
            }
        }
        visiting.remove(&id);
        Some(result)
    }

    fn call_substitutions(
        &self,
        callable: &Callable,
        returned: &ReturnedCallables,
        arguments: &Arguments,
    ) -> (FxHashMap<IdentifierId, Values>, FxHashMap<IdentifierId, Properties>) {
        let mut arguments = {
            let mut bound = callable.bound_arguments.clone();
            bound.append(arguments);
            bound
        };
        for values in arguments.elements.values_mut() {
            for identity in values.identities.clone() {
                values.extend(&self.values(identity));
            }
        }
        let mut substitutions = FxHashMap::default();
        let mut rest_arguments = FxHashMap::default();
        for &binding in &returned.external_contexts {
            if let Some(identities) = callable.context_values.get(&binding) {
                let mut values = Values::default();
                values.identities.clone_from(identities);
                for &id in identities {
                    values.extend(&self.values(id));
                }
                substitutions.insert(binding, values);
            }
        }
        for (index, &(parameter, is_rest)) in returned.params.iter().enumerate() {
            if is_rest {
                let elements: Properties = arguments
                    .elements
                    .iter()
                    .filter_map(|(key, value)| {
                        let key = match key {
                            Some(key) => Some(array_index(key)?.checked_sub(index)?.to_string()),
                            None => None,
                        };
                        Some((key, value.clone()))
                    })
                    .collect();
                let mut values = Values::default();
                for element in elements.values() {
                    values.extend(element);
                }
                substitutions.insert(parameter, values);
                rest_arguments.insert(parameter, elements);
            } else {
                let mut values =
                    arguments.elements.get(&Some(index.to_string())).cloned().unwrap_or_default();
                if let Some(unknown) = arguments.elements.get(&None) {
                    values.extend(unknown);
                }
                substitutions.insert(parameter, values);
            }
        }
        (substitutions, rest_arguments)
    }

    fn call_arguments(&self, instruction: &Instruction<'_>) -> (Arguments, Option<IdentifierId>) {
        let mut binding = None;
        let arguments = match &instruction.value {
            InstructionValue::CallExpression { args, .. }
            | InstructionValue::NewExpression { args, .. } => self.arguments(args),
            InstructionValue::MethodCall { receiver, property, args, .. } => {
                match self.builtin_function_methods.get(&property.identifier).map(String::as_str) {
                    Some("bind") => {
                        binding = Some(receiver.identifier);
                        self.arguments(args).skip(1)
                    }
                    Some("call") => self.arguments(args).skip(1),
                    Some("next") => Arguments::default(),
                    Some("apply") => args.get(1).map_or_else(Arguments::default, |argument| {
                        let place = match argument {
                            PlaceOrSpread::Place(place) => place,
                            PlaceOrSpread::Spread(spread) => &spread.place,
                        };
                        Arguments {
                            elements: self.properties(place.identifier),
                            length: self.array_length(&self.values(place.identifier)),
                        }
                    }),
                    _ => self.arguments(args),
                }
            }
            InstructionValue::TaggedTemplateExpression { subexprs, .. } => Arguments {
                elements: subexprs
                    .iter()
                    .enumerate()
                    .map(|(index, place)| {
                        (Some((index + 1).to_string()), self.values(place.identifier))
                    })
                    .collect(),
                length: Some(subexprs.len() + 1),
            },
            _ => Arguments::default(),
        };
        (arguments, binding)
    }

    fn call_result(&mut self, instruction: &Instruction<'_>) {
        let (arguments, binding) = self.call_arguments(instruction);
        if let Some(receiver) = binding {
            let mut values = self.values(receiver);
            // Bound arguments snapshot their values. Keep identities here so
            // binding a function to itself cannot build recursive summaries.
            let mut arguments = arguments;
            for values in arguments.elements.values_mut() {
                values.contexts.clear();
                values.iterators.clear();
            }
            for callable in values.contexts.values_mut() {
                callable.bound_arguments.append(&arguments);
            }
            values.identities = FxHashSet::from_iter([instruction.lvalue.identifier]);
            self.values.insert(instruction.lvalue.identifier, values);
            return;
        }
        let invoked = self.invoked(&instruction.value);
        let mut result = Values::default();
        for callable in invoked.contexts.values() {
            if callable.generator {
                // Calling a generator snapshots its arguments, but neither runs
                // its body nor observes its live captured bindings yet.
                let mut callable = callable.clone();
                let mut arguments = arguments.clone();
                for values in arguments.elements.values_mut() {
                    values.contexts.clear();
                    values.iterators.clear();
                }
                callable.bound_arguments.append(&arguments);
                let id = instruction.lvalue.identifier;
                let iterator = Values {
                    identities: FxHashSet::from_iter([id]),
                    iterators: FxHashMap::from_iter([(id, callable)]),
                    ..Values::default()
                };
                result.extend(&iterator);
                continue;
            }
            result.extend(&self.returned_values(callable, &arguments, &mut FxHashSet::default()));
        }
        let callees: FxHashSet<_> = invoked
            .identities
            .iter()
            .filter(|id| self.projections.contains_key(id))
            .copied()
            .collect();
        if !callees.is_empty() {
            self.deferred_calls
                .insert(instruction.lvalue.identifier, DeferredCall { callees, arguments });
            self.projections.entry(instruction.lvalue.identifier).or_default();
            result.identities.insert(instruction.lvalue.identifier);
        }
        self.values.insert(instruction.lvalue.identifier, result);
    }

    fn returned_values(
        &self,
        callable: &Callable,
        arguments: &Arguments,
        visiting: &mut FxHashSet<IdentifierId>,
    ) -> Values {
        // Async calls produce promises, not the eventual returned callable.
        if callable.is_async {
            return Values::default();
        }
        let Some(returned) = &callable.returned else {
            return Values::default();
        };
        let (substitutions, rest_arguments) =
            self.call_substitutions(callable, returned, arguments);
        self.substitute_values(
            &returned.values,
            returned,
            &substitutions,
            &rest_arguments,
            visiting,
        )
    }

    fn substitute_values(
        &self,
        original: &Values,
        returned: &ReturnedCallables,
        substitutions: &FxHashMap<IdentifierId, Values>,
        rest_arguments: &FxHashMap<IdentifierId, Properties>,
        visiting: &mut FxHashSet<IdentifierId>,
    ) -> Values {
        let mut values = original.clone();
        // Returning a parameter snapshots the selected callable; a newly
        // returned closure retains the caller's live binding information.
        for identity in &original.identities {
            if let Some(selected) = self.substitute(
                *identity,
                substitutions,
                &returned.projections,
                &returned.deferred_calls,
                rest_arguments,
                visiting,
            ) {
                values.identities.remove(identity);
                values.extend(&selected);
            }
        }
        for closure in values.contexts.values_mut().chain(values.iterators.values_mut()) {
            for (&binding, values) in Rc::make_mut(&mut closure.context_values).iter_mut() {
                if returned.external_contexts.contains(&binding)
                    && let Some(current) = substitutions.get(&binding)
                {
                    *values = current.identities.clone();
                } else {
                    *values = values
                        .iter()
                        .flat_map(|value| {
                            if let Some(elements) = rest_arguments.get(value)
                                && let Some(members) = closure.capture_members.get(&binding)
                            {
                                return members
                                    .iter()
                                    .filter(|member| array_index(member).is_some())
                                    .flat_map(|member| {
                                        elements
                                            .get(&Some(member.clone()))
                                            .into_iter()
                                            .chain(elements.get(&None))
                                    })
                                    .flat_map(|value| value.identities.iter().copied())
                                    .collect();
                            }
                            self.substitute(
                                *value,
                                substitutions,
                                &returned.projections,
                                &returned.deferred_calls,
                                rest_arguments,
                                visiting,
                            )
                            .map_or_else(
                                || FxHashSet::from_iter([*value]),
                                |values| values.identities,
                            )
                        })
                        .collect();
                }
            }
        }
        values
    }

    /// A write to some unknown index of an array, as `push`, `unshift`, `fill`
    /// and `splice` do. Named properties such as own methods keep their values.
    fn store_index(&mut self, objects: &Values, values: &Values) {
        for &object in &objects.identities {
            self.array_lengths.remove(&object);
            let properties = self.properties.entry(object).or_default();
            for (key, property) in properties.iter_mut() {
                if key.as_ref().is_none_or(|key| array_index(key).is_some()) {
                    property.extend(values);
                }
            }
            properties.entry(None).or_default().extend(values);
        }
        self.refresh_pending(objects, None);
    }

    /// A delete leaves the key without values and, unlike a store of an
    /// untracked value, makes the own property absent, so a built-in method of
    /// the same name runs again.
    fn delete(&mut self, object: IdentifierId, key: Option<String>) {
        self.store(object, key.clone(), Values::default());
        for object in self.values(object).identities {
            if let Some(keys) = self.present.get_mut(&object) {
                match &key {
                    Some(key) => {
                        keys.remove(key);
                    }
                    None => keys.clear(),
                }
            }
        }
    }

    /// Whether `objects` names one runtime object, so a write may replace its
    /// old contents. A Map entry pair summarizes every pair materialized from
    /// that Map at that site, so writes through it are always weak.
    fn singleton(&self, objects: &Values) -> bool {
        objects.identities.len() == 1
            && objects.identities.iter().all(|&id| !self.entry_ids.contains(id))
    }

    fn store(&mut self, object: IdentifierId, key: Option<String>, values: Values) {
        let objects = self.values(object);
        self.store_into(&objects, key, values);
    }

    fn store_into(&mut self, objects: &Values, key: Option<String>, values: Values) {
        let definite = self.singleton(objects) && key.is_some();
        for &object in &objects.identities {
            self.array_lengths.remove(&object);
            if key.is_none() {
                let properties = self.properties.entry(object).or_default();
                for property in properties.values_mut() {
                    property.extend(&values);
                }
                properties.entry(None).or_default().extend(&values);
                if self.arrays.contains(&object) {
                    self.named_unknown.entry(object).or_default().extend(&values);
                }
            } else if let (true, Some(name)) = (definite, &key) {
                self.properties.entry(object).or_default().insert(key.clone(), values.clone());
                self.present.entry(object).or_default().insert(name.clone());
            } else {
                let properties = self.properties.get(&object);
                let fallback = properties
                    .and_then(|properties| self.unknown_fallback(object, properties, &key))
                    .cloned()
                    .unwrap_or_default();
                let properties = self.properties.entry(object).or_default();
                properties.entry(key.clone()).or_insert(fallback).extend(&values);
            }
        }
        self.refresh_pending(objects, key.as_ref());
    }

    fn refresh_pending(&mut self, objects: &Values, key: Option<&String>) {
        // Update only pending reads touched by this write. Recomputing every
        // capture after a CFG join would mix in stores from non-invoking paths.
        let affected: Vec<_> = self
            .pending
            .iter()
            .filter_map(|(&binding, capture)| {
                objects
                    .identities
                    .iter()
                    .any(|object| {
                        capture.dependencies.get(object).is_some_and(|members| {
                            key.is_none_or(|key| {
                                members.as_ref().is_none_or(|members| members.contains(key))
                            })
                        })
                    })
                    .then_some(binding)
            })
            .collect();
        for binding in affected {
            let mut capture = self.pending.remove(&binding).unwrap();
            self.refresh_capture(&mut capture);
            self.pending.insert(binding, capture);
        }
    }

    fn destructure(&mut self, pattern: &Pattern<'_>, source: IdentifierId) {
        match pattern {
            Pattern::Object(pattern) => {
                let mut excluded = FxHashSet::default();
                for property in &pattern.properties {
                    match property {
                        ObjectPropertyOrSpread::Property(property) => {
                            let key = self.key(&property.key);
                            if let Some(key) = &key {
                                excluded.insert(key.clone());
                            }
                            self.load(source, key, property.place.identifier);
                        }
                        ObjectPropertyOrSpread::Spread(spread) => {
                            let mut properties = self.properties(source);
                            properties.retain(|key, _| {
                                key.as_ref().is_none_or(|key| !excluded.contains(key))
                            });
                            self.allocate(spread.place.identifier);
                            self.properties.insert(spread.place.identifier, properties);
                        }
                    }
                }
            }
            Pattern::Array(pattern) => {
                for (index, element) in pattern.items.iter().enumerate() {
                    match element {
                        ArrayPatternElement::Place(place) => {
                            self.load(source, Some(index.to_string()), place.identifier)
                        }
                        ArrayPatternElement::Spread(spread) => {
                            let properties = self
                                .properties(source)
                                .into_iter()
                                .filter_map(|(key, values)| {
                                    let key = match key {
                                        Some(key) => {
                                            Some(array_index(&key)?.checked_sub(index)?.to_string())
                                        }
                                        None => None,
                                    };
                                    Some((key, values))
                                })
                                .collect();
                            self.allocate(spread.place.identifier);
                            self.properties.insert(spread.place.identifier, properties);
                        }
                        ArrayPatternElement::Hole => {}
                    }
                }
            }
        }
    }

    fn apply(
        &mut self,
        instruction: &Instruction<'_>,
        hoisted: &FxHashMap<IdentifierId, IdentifierId>,
        async_functions: &FxHashSet<IdentifierId>,
        captured_contexts: &Captures,
        function_returns: &FxHashMap<IdentifierId, ReturnedCallables>,
        captured_bindings: &FxHashSet<IdentifierId>,
    ) {
        let id = instruction.lvalue.identifier;
        match &instruction.value {
            InstructionValue::FunctionExpression { .. } | InstructionValue::ObjectMethod { .. } => {
                self.allocate(id);
                let mut contexts = self.context_values.clone();
                for &capture in captured_contexts.get(&id).into_iter().flatten() {
                    contexts.entry(capture).or_insert_with(|| FxHashSet::from_iter([capture]));
                }
                self.values.get_mut(&id).unwrap().contexts.insert(
                    id,
                    Callable {
                        is_async: async_functions.contains(&id),
                        generator: function_returns
                            .get(&id)
                            .is_some_and(|summary| summary.generator),
                        deferred_captures: function_returns
                            .get(&id)
                            .map(|summary| summary.deferred_captures.clone())
                            .unwrap_or_default(),
                        context_values: Rc::new(contexts),
                        returned: function_returns.get(&id).cloned().map(Rc::new),
                        snapshot_captures: function_returns
                            .get(&id)
                            .map(|summary| summary.snapshot_captures.clone())
                            .unwrap_or_default(),
                        bound_arguments: Arguments::default(),
                        eager_captures: function_returns
                            .get(&id)
                            .map(|summary| summary.eager_captures.clone())
                            .unwrap_or_default(),
                        capture_reads: function_returns
                            .get(&id)
                            .map(|summary| summary.capture_reads.clone())
                            .unwrap_or_default(),
                        capture_members: function_returns
                            .get(&id)
                            .map(|summary| summary.capture_members.clone())
                            .unwrap_or_default(),
                    },
                );
            }
            InstructionValue::Primitive { value, .. } => {
                let key = match value {
                    PrimitiveValue::String(value) => {
                        value.as_str().map(|text| (PrimitiveKind::String, text.to_string()))
                    }
                    PrimitiveValue::Number(value) => {
                        Some((PrimitiveKind::Number, value.to_string()))
                    }
                    PrimitiveValue::BigInt { value, .. } => {
                        Some((PrimitiveKind::BigInt, value.as_str().to_string()))
                    }
                    PrimitiveValue::Boolean(value) => {
                        Some((PrimitiveKind::Boolean, value.to_string()))
                    }
                    PrimitiveValue::Null => Some((PrimitiveKind::Null, "null".to_string())),
                    PrimitiveValue::Undefined => {
                        Some((PrimitiveKind::Undefined, "undefined".to_string()))
                    }
                }
                .map(|(kind, text)| PrimitiveKey { kind, text });
                if let Some(key) = key {
                    self.keys.insert(id, key);
                } else {
                    self.keys.remove(&id);
                }
                self.values.insert(id, Values::default());
            }
            InstructionValue::LoadLocal { place, .. }
            | InstructionValue::LoadContext { place, .. } => self.copy(place.identifier, id),
            InstructionValue::TypeCastExpression { value, .. } => self.copy(value.identifier, id),
            InstructionValue::StoreLocal { lvalue, value, .. }
            | InstructionValue::StoreContext { lvalue, value, .. } => {
                if matches!(instruction.value, InstructionValue::StoreContext { .. })
                    && lvalue.kind == InstructionKind::Function
                    && hoisted.contains_key(&lvalue.place.identifier)
                {
                    return;
                }
                self.copy(value.identifier, lvalue.place.identifier);
                self.copy(value.identifier, id);
                if matches!(instruction.value, InstructionValue::StoreContext { .. })
                    || captured_bindings.contains(&lvalue.place.identifier)
                {
                    self.update_context(lvalue.place.identifier);
                }
            }
            InstructionValue::DeclareContext { lvalue, .. } => {
                if lvalue.kind == InstructionKind::HoistedFunction
                    && let Some(value) = hoisted.get(&lvalue.place.identifier)
                {
                    let mut values = Values::single(*value);
                    values.contexts.insert(
                        *value,
                        Callable {
                            is_async: async_functions.contains(value),
                            generator: function_returns
                                .get(value)
                                .is_some_and(|summary| summary.generator),
                            deferred_captures: function_returns
                                .get(value)
                                .map(|summary| summary.deferred_captures.clone())
                                .unwrap_or_default(),
                            context_values: Rc::new(self.context_values.clone()),
                            returned: function_returns.get(value).cloned().map(Rc::new),
                            snapshot_captures: function_returns
                                .get(value)
                                .map(|summary| summary.snapshot_captures.clone())
                                .unwrap_or_default(),
                            bound_arguments: Arguments::default(),
                            eager_captures: function_returns
                                .get(value)
                                .map(|summary| summary.eager_captures.clone())
                                .unwrap_or_default(),
                            capture_reads: function_returns
                                .get(value)
                                .map(|summary| summary.capture_reads.clone())
                                .unwrap_or_default(),
                            capture_members: function_returns
                                .get(value)
                                .map(|summary| summary.capture_members.clone())
                                .unwrap_or_default(),
                        },
                    );
                    self.values.insert(lvalue.place.identifier, values);
                } else {
                    self.values.insert(lvalue.place.identifier, Values::default());
                }
                self.keys.remove(&lvalue.place.identifier);
                self.update_context(lvalue.place.identifier);
            }
            InstructionValue::ObjectExpression { properties, .. } => {
                self.allocate(id);
                self.properties.insert(id, Properties::default());
                for property in properties {
                    match property {
                        ObjectPropertyOrSpread::Property(property) => {
                            self.store(
                                id,
                                self.key(&property.key),
                                self.values(property.place.identifier),
                            );
                        }
                        ObjectPropertyOrSpread::Spread(spread) => {
                            let source = spread.place.identifier;
                            let sources = self.values(source);
                            let exact = sources.identities.len() == 1
                                && sources.identities.contains(&source)
                                && self.exact_objects.contains(&source);
                            self.assign(&Values::single(id), &sources, !exact);
                        }
                    }
                }
                if properties
                    .iter()
                    .all(|property| matches!(property, ObjectPropertyOrSpread::Property(_)))
                {
                    self.exact_objects.insert(id);
                }
            }
            InstructionValue::ArrayExpression { elements, .. } => {
                self.allocate(id);
                self.arrays.insert(id);
                let mut contents = Arguments::default();
                for element in elements {
                    let next = match element {
                        ArrayElement::Place(place) => Arguments {
                            elements: FxHashMap::from_iter([(
                                Some("0".to_string()),
                                self.values(place.identifier),
                            )]),
                            length: Some(1),
                        },
                        ArrayElement::Spread(spread) => self.spread(spread.place.identifier),
                        ArrayElement::Hole => {
                            Arguments { elements: Properties::default(), length: Some(1) }
                        }
                    };
                    contents.append(&next);
                }
                self.properties.insert(id, contents.elements);
                if let Some(length) = contents.length {
                    self.array_lengths.insert(id, length);
                } else {
                    self.array_lengths.remove(&id);
                }
            }
            InstructionValue::PropertyLoad { object, property, .. } => {
                self.load(object.identifier, Some(property.to_string()), id);
                self.members.insert(id, property.to_string());
            }
            InstructionValue::LoadGlobal { binding: NonLocalBinding::Global { name }, .. } => {
                self.globals.insert(id, name.to_string());
            }
            InstructionValue::ComputedLoad { object, property, .. } => {
                self.load(object.identifier, self.property_key(property.identifier), id)
            }
            InstructionValue::PropertyStore { object, property, value, .. } => self.store(
                object.identifier,
                Some(property.to_string()),
                self.values(value.identifier),
            ),
            InstructionValue::ComputedStore { object, property, value, .. } => self.store(
                object.identifier,
                self.property_key(property.identifier),
                self.values(value.identifier),
            ),
            InstructionValue::PropertyDelete { object, property, .. } => {
                self.delete(object.identifier, Some(property.to_string()));
            }
            InstructionValue::ComputedDelete { object, property, .. } => {
                self.delete(object.identifier, self.property_key(property.identifier));
            }
            InstructionValue::Destructure { lvalue, value, .. } => {
                self.destructure(&lvalue.pattern, value.identifier);
                self.copy(value.identifier, id);
            }
            InstructionValue::PrefixUpdateContext { lvalue, .. }
            | InstructionValue::PostfixUpdateContext { lvalue, .. } => {
                self.values.insert(lvalue.identifier, Values::default());
                self.keys.remove(&lvalue.identifier);
                self.update_context(lvalue.identifier);
            }
            InstructionValue::CallExpression { .. }
            | InstructionValue::NewExpression { .. }
            | InstructionValue::MethodCall { .. }
            | InstructionValue::TaggedTemplateExpression { .. } => self.call_result(instruction),
            _ => {}
        }
    }
}

/// Resolve async identity at each value-producing instruction. Heap properties
/// use reaching values across the CFG: a load snapshots the value before later
/// writes, and a definite store replaces the old member. Containment never makes
/// a container itself callable.
pub(super) fn infer_async_callable_contexts(
    func: &HirFunction,
    async_functions: &FxHashSet<IdentifierId>,
    captured_contexts: &Captures,
    hoisted: &FxHashMap<IdentifierId, IdentifierId>,
    function_returns: &FxHashMap<IdentifierId, ReturnedCallables>,
    env: &Environment<'_>,
) -> AsyncCallables {
    let mut result = AsyncCallables {
        identifiers: FxHashSet::default(),
        invocations: FxHashMap::default(),
        returning_callbacks: FxHashMap::default(),
        returned: ReturnedCallables::default(),
        pending_before: FxHashMap::default(),
        pending_at_exit: FxHashMap::default(),
    };
    let parameters = parameter_bindings(func, env);
    let captured_bindings = captured_contexts.values().flatten().copied().collect();
    let mut states: FxHashMap<BlockId, CallableState> = FxHashMap::default();
    let entry_ids = EntryIds::new(env.identifiers.len());
    loop {
        let mut changed = false;
        for (&block_id, block) in &func.body.blocks {
            let mut predecessors =
                block.preds.iter().filter_map(|predecessor| states.get(predecessor));
            let mut state = predecessors.next().cloned().unwrap_or_else(|| {
                let mut state =
                    CallableState { entry_ids: entry_ids.clone(), ..CallableState::default() };
                for place in
                    func.context.iter().chain(func.params.iter().map(|param| match param {
                        ParamPattern::Place(place) => place,
                        ParamPattern::Spread(spread) => &spread.place,
                    }))
                {
                    state.allocate(place.identifier);
                    state.projections.entry(place.identifier).or_default();
                    state
                        .context_values
                        .insert(place.identifier, FxHashSet::from_iter([place.identifier]));
                }
                state
            });
            for predecessor in predecessors {
                state.merge(predecessor);
            }
            for phi in &block.phis {
                let mut values = Values::default();
                let mut keys = Vec::new();
                for (predecessor, operand) in &phi.operands {
                    if let Some(predecessor) = states.get(predecessor) {
                        values.extend(&predecessor.values(operand.identifier));
                        keys.push(predecessor.keys.get(&operand.identifier));
                    }
                }
                state.values.insert(phi.place.identifier, values);
                if let Some(Some(key)) = keys.first()
                    && keys.iter().all(|other| *other == Some(*key))
                {
                    state.keys.insert(phi.place.identifier, (*key).clone());
                } else {
                    state.keys.remove(&phi.place.identifier);
                }
            }
            for &instruction_id in &block.instructions {
                let instruction = &func.instructions[instruction_id.index()];
                if let InstructionValue::LoadLocal { place, .. }
                | InstructionValue::LoadContext { place, .. } = &instruction.value
                    && parameters.contains(&place.identifier)
                {
                    result
                        .returned
                        .parameter_values
                        .entry(place.identifier)
                        .or_default()
                        .extend(state.values(place.identifier).identities);
                }
                let mut invocations = Vec::new();
                let (arguments, _) = state.call_arguments(instruction);
                let calls: Vec<_> = [
                    Call { callees: state.invoked(&instruction.value), arguments: Some(arguments) },
                    Call {
                        callees: state.invoked_callbacks(&instruction.value, env, false),
                        arguments: state.callback_arguments(&instruction.value, env),
                    },
                ]
                .into_iter()
                .filter(|call| {
                    call.callees.identities.iter().any(|id| state.projections.contains_key(id))
                        || call.callees.contexts.values().any(|callable| {
                            !callable.generator
                                && (callable.is_async
                                    || callable
                                        .returned
                                        .as_ref()
                                        .is_some_and(|summary| !summary.calls.is_empty()))
                        })
                })
                .collect();
                for call in &calls {
                    state.execute_call(call, &mut FxHashSet::default(), &mut invocations);
                }
                if calls.is_empty() {
                    result.returned.calls.remove(&instruction_id);
                } else {
                    result.returned.calls.insert(instruction_id, calls);
                }
                let returning = state.invoked_callbacks(&instruction.value, env, true);
                let mut identifiers = returning.identities;
                identifiers.extend(returning.contexts.keys().copied());
                if identifiers.is_empty() {
                    result.returning_callbacks.remove(&instruction_id);
                } else {
                    result.returning_callbacks.insert(instruction_id, identifiers);
                }
                result.invocations.insert(instruction_id, invocations);
                if matches!(instruction.value, InstructionValue::Await { .. }) {
                    result.pending_before.insert(instruction_id, state.pending_observation());
                }
                state.apply(
                    instruction,
                    hoisted,
                    async_functions,
                    captured_contexts,
                    function_returns,
                    &captured_bindings,
                );
                match state.method_override(instruction) {
                    MethodOverride::Definite => {}
                    MethodOverride::Possible => {
                        // Either method may run: join the whole heap after the
                        // stored method with the heap after the built-in.
                        let custom = state.clone();
                        state.update_builtins(instruction, env);
                        state.merge(&custom);
                    }
                    MethodOverride::None => state.update_builtins(instruction, env),
                }
            }
            if matches!(block.terminal, Terminal::Return { .. } | Terminal::Throw { .. }) {
                result.pending_at_exit.insert(block_id, state.pending_observation());
            }
            if states.get(&block_id) != Some(&state) {
                states.insert(block_id, state);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for state in states.values() {
        for (&identifier, values) in &state.values {
            if values.contexts.values().any(|callable| callable.is_async) {
                result.identifiers.insert(identifier);
            }
        }
    }
    for (&block_id, block) in &func.body.blocks {
        if let Terminal::Return { value, .. } = &block.terminal {
            result.returned.values.extend(&states[&block_id].values(value.identifier));
        }
    }
    result.returned.generator = func.generator;
    result.returned.external_contexts.extend(func.context.iter().map(|place| place.identifier));
    result.returned.params.extend(func.params.iter().map(|param| match param {
        ParamPattern::Place(place) => (place.identifier, false),
        ParamPattern::Spread(spread) => (spread.place.identifier, true),
    }));
    result.returned.capture_members = infer_capture_members(func, env, true);
    result.returned.capture_reads = infer_capture_members(func, env, false);
    (
        result.returned.eager_captures,
        result.returned.deferred_captures,
        result.returned.snapshot_captures,
    ) = infer_capture_phases(
        func,
        &result.invocations,
        &result.returned.capture_reads,
        &parameters,
    );
    for calls in states
        .values()
        .map(|state| &state.deferred_calls)
        .chain(function_returns.values().map(|summary| &summary.deferred_calls))
    {
        for (&id, call) in calls {
            if let Some(into) = result.returned.deferred_calls.get_mut(&id) {
                into.merge(call);
            } else {
                result.returned.deferred_calls.insert(id, call.clone());
            }
        }
    }
    for projections in states
        .values()
        .map(|state| &state.projections)
        .chain(function_returns.values().map(|summary| &summary.projections))
    {
        for (&id, sources) in projections {
            result.returned.projections.entry(id).or_default().extend(sources.iter().cloned());
        }
    }
    result
}
