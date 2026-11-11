//! Gives each sloppy-mode block function that assigns a function-scoped
//! binding of the same name (Annex B.3.3) that binding's symbol before
//! mangling.
//!
//! Semantic analysis keeps such a function in its block scope when the
//! enclosing var scope already binds the name. When the block runs, the
//! function object is still copied into that binding, by name. The mangler
//! reuses slots by liveness and treats the block function as a separate
//! block-local binding, so it can rename the two differently. The copy then
//! misses the binding and overwrites whichever binding took the block
//! function's new name. One shared symbol gives both one name.
//!
//! `keep_names` keeps a symbol by its first declaration, which is no longer
//! the function after aliasing, so the caller reserves the returned names.
//! Names that block an Annex B copy are always reserved so mangling cannot
//! introduce the copy by separating a function from its lexical barrier.

use oxc_ast::AstKind;
use oxc_semantic::{AstNodes, Scoping, Semantic};
use oxc_str::{CompactStr, Ident};
use oxc_syntax::{
    node::NodeId,
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};
use rustc_hash::FxHashSet;

pub fn alias_annex_b_block_functions(
    semantic: &mut Semantic<'_>,
    reserved: &mut FxHashSet<CompactStr>,
) -> std::vec::Vec<CompactStr> {
    let mut names = std::vec![];
    if semantic.source_type().is_strict() {
        return names;
    }
    let (scoping, nodes) = semantic.scoping_mut_and_nodes();
    let mut seen = FxHashSet::default();
    let aliases: std::vec::Vec<_> = nodes
        .iter()
        .filter_map(|node| {
            let AstKind::Function(function) = node.kind() else { return None };
            if !function.is_function_declaration() || function.r#async || function.generator {
                return None;
            }
            let block_function = function.id.as_ref()?.symbol_id.get()?;
            let binding =
                annex_b_binding(scoping, nodes, block_function, node.scope_id(), reserved)?;
            (block_function != binding && seen.insert(block_function))
                .then_some((block_function, binding))
        })
        .collect();
    for (block_function, binding) in aliases {
        if let Some(name) = alias(scoping, nodes, block_function, binding) {
            names.push(CompactStr::from(name.as_str()));
        }
    }
    names
}

/// The function-scoped binding that `symbol_id`, a sloppy-mode block-level
/// function declaration kept in its block scope, assigns under Annex B.3.3.
fn annex_b_binding(
    scoping: &Scoping,
    nodes: &AstNodes<'_>,
    symbol_id: SymbolId,
    block_scope_id: ScopeId,
    reserved: &mut FxHashSet<CompactStr>,
) -> Option<SymbolId> {
    if !is_plain_function(scoping.symbol_flags(symbol_id)) {
        return None;
    }
    let block_scope_flags = scoping.scope_flags(block_scope_id);
    if block_scope_flags.is_var() || block_scope_flags.is_strict_mode() {
        return None;
    }
    let name = scoping.symbol_ident(symbol_id);
    if name == "arguments" {
        reserved.insert(CompactStr::from(name.as_str()));
        return None;
    }
    // Intervening lexical bindings block the Annex B copy. Plain functions and
    // simple catch parameters do not; destructured catch parameters remain lexical.
    let mut var_scope_id: Option<ScopeId> = None;
    for scope_id in scoping.scope_ancestors(block_scope_id).skip(1) {
        if scoping.scope_flags(scope_id).is_var() {
            var_scope_id = Some(scope_id);
            break;
        }
        if let Some(enclosing) = scoping.get_binding(scope_id, name) {
            let flags = scoping.symbol_flags(enclosing);
            if !is_plain_function(flags)
                && !flags.contains(SymbolFlags::CatchVariable | SymbolFlags::FunctionScopedVariable)
            {
                // Keep both names so renaming cannot introduce a previously blocked copy.
                reserved.insert(CompactStr::from(name.as_str()));
                return None;
            }
        }
    }
    let binding = scoping.get_binding(var_scope_id?, name)?;
    let binding_flags = scoping.symbol_flags(binding);
    // It also skips a name lexically declared in the var scope and, in a
    // function, a parameter's name.
    if binding_flags.intersects(SymbolFlags::BlockScopedVariable | SymbolFlags::Class)
        || !binding_flags.intersects(SymbolFlags::Function | SymbolFlags::FunctionScopedVariable)
        || scoping.symbol_declarations(binding).any(|node_id| is_parameter(nodes, node_id))
    {
        reserved.insert(CompactStr::from(name.as_str()));
        return None;
    }
    Some(binding)
}

/// A declared function that is neither async nor a generator.
fn is_plain_function(flags: SymbolFlags) -> bool {
    flags.contains(SymbolFlags::Function)
        && !flags
            .intersects(SymbolFlags::FunctionExpression | SymbolFlags::AsyncOrGeneratorFunction)
}

fn is_parameter(nodes: &AstNodes<'_>, node_id: NodeId) -> bool {
    for kind in nodes.ancestor_kinds(node_id) {
        match kind {
            AstKind::FormalParameters(_) => return true,
            AstKind::Function(_)
            | AstKind::ArrowFunctionExpression(_)
            | AstKind::FunctionBody(_)
            | AstKind::StaticBlock(_)
            | AstKind::Program(_) => return false,
            _ => {}
        }
    }
    false
}

fn alias<'a>(
    scoping: &mut Scoping,
    nodes: &AstNodes<'a>,
    block_function: SymbolId,
    binding: SymbolId,
) -> Option<Ident<'a>> {
    let flags = scoping.symbol_flags(block_function);
    let declarations: std::vec::Vec<_> = scoping.symbol_declarations(block_function).collect();
    let mut name = None;
    for declaration in declarations {
        let AstKind::Function(function) = nodes.kind(declaration) else { continue };
        let Some(id) = &function.id else { continue };
        id.symbol_id.set(Some(binding));
        name = Some(id.name);
        scoping.add_symbol_redeclaration(binding, flags, declaration, id.span);
    }
    while let Some(&reference_id) = scoping.get_resolved_reference_ids(block_function).first() {
        scoping.delete_resolved_reference(block_function, reference_id);
        scoping.get_reference_mut(reference_id).set_symbol_id(binding);
        scoping.add_resolved_reference(binding, reference_id);
    }
    let block_scope_id = scoping.symbol_scope_id(block_function);
    if let Some(name) = name {
        scoping.remove_binding(block_scope_id, name);
    }
    name
}
