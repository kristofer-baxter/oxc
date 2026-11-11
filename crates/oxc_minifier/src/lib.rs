//! # Oxc Minifier
//!
//! High-performance JavaScript minifier focused on maximum compression.
//! Transform TypeScript, TSX, and JSX to JavaScript before minification.
//!
//! ## Overview
//!
//! The Oxc minifier applies a comprehensive set of optimizations to JavaScript code
//! to achieve the smallest possible output while maintaining correctness. It draws
//! inspiration from industry-leading minifiers like Closure Compiler, Terser, esbuild,
//! and SWC.
//!
//! ## Features
//!
//! - **Compression**: Repeated passes expose further optimization opportunities
//! - **Comprehensive Optimizations**: Constant folding, dead code removal, and syntax simplification
//! - **Correctness Testing**: Selected test262/Babel idempotency checks and Node.js runtime tests
//!   under the [documented assumptions](https://github.com/oxc-project/oxc/blob/main/crates/oxc_minifier/docs/ASSUMPTIONS.md)
//! - **Fast**: Efficient algorithms and arena allocation for performance
//!
//! ## Example
//!
//! ```rust
//! use oxc_minifier::{Minifier, MinifierOptions};
//! use oxc_allocator::Allocator;
//! use oxc_parser::Parser;
//! use oxc_span::SourceType;
//!
//! let allocator = Allocator::default();
//! let source_text = "const x = 1 + 1; console.log(x);";
//! let source_type = SourceType::mjs();
//! let ret = Parser::new(&allocator, source_text, source_type).parse();
//! let mut program = ret.program;
//!
//! let options = MinifierOptions::default();
//! let minifier = Minifier::new(options);
//! let result = minifier.minify(&allocator, &mut program);
//! ```
//!
//! ## Architecture
//!
//! The minifier consists of:
//! - **Compressor**: Orchestrates the optimization pipeline
//! - **Peephole Optimizations**: Individual transformation passes
//! - **Mangler**: Variable renaming for size reduction
//!
//! See the [crate documentation](https://github.com/oxc-project/oxc/tree/main/crates/oxc_minifier) for more details.

mod annex_b_alias;
mod compression_pass;
mod compressor;
pub(crate) mod generated;
mod is_terminated;
mod keep_var;
mod minifier_traverse;
mod options;
mod peephole;
mod property_mangler;
mod state;
mod symbol_liveness;
mod symbol_metadata;
mod symbol_state;
mod symbol_value;
mod traverse_context;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_index::IndexVec;
use oxc_mangler::Mangler;
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_str::CompactStr;
use oxc_syntax::class::ClassId;
use rustc_hash::FxHashMap;

use crate::state::CompressionMode;

pub use oxc_mangler::{MangleOptions, MangleOptionsKeepNames};
pub use property_mangler::{
    InvalidManglePropertyCacheTarget, ManglePropertiesOptions, ManglePropertyCache,
    PropertyMangleCollection, PropertyMangler, PropertyMapping,
    is_valid_property_mangle_cache_target,
};

pub(crate) use crate::generated::traverse::Traverse;
#[doc(hidden)]
pub(crate) use crate::traverse_context::MinifierTraverseCtx as TraverseCtx;
pub(crate) use crate::traverse_context::ReusableMinifierTraverseCtx as ReusableTraverseCtx;
pub use crate::{compressor::Compressor, options::*};

/// Whether property folding must account for possible writes to `Object.prototype`.
pub fn program_may_mutate_object_prototype(program: &Program<'_>) -> bool {
    peephole::PeepholeOptimizations::program_may_mutate_object_prototype(program)
}

#[derive(Debug, Clone)]
pub struct MinifierOptions {
    pub mangle: Option<MangleOptions>,
    /// Property-name mangling for the current [`Program`].
    ///
    /// This option does not coordinate mappings across programs. Bundlers that need cross-file
    /// consistency must collect all programs with [`PropertyMangler`], assign once, and rewrite
    /// each program exactly once before emitting chunks.
    pub mangle_properties: Option<ManglePropertiesOptions>,
    pub compress: Option<CompressOptions>,
}

impl Default for MinifierOptions {
    fn default() -> Self {
        Self {
            mangle: Some(MangleOptions::default()),
            mangle_properties: None,
            compress: Some(CompressOptions::default()),
        }
    }
}

pub struct MinifierReturn {
    pub scoping: Option<Scoping>,

    /// A vector where each element corresponds to a class in declaration order.
    /// Each element is a mapping from original private member names to their mangled names.
    pub class_private_mappings: Option<IndexVec<ClassId, FxHashMap<String, CompactStr>>>,

    /// Updated property-name cache when property mangling ran.
    pub property_mangle_cache: Option<ManglePropertyCache>,

    /// Total number of iterations ran. Useful for debugging performance issues.
    pub iterations: u8,
}

pub struct Minifier {
    options: MinifierOptions,
}

impl<'a> Minifier {
    pub fn new(options: MinifierOptions) -> Self {
        Self { options }
    }

    pub fn minify(self, allocator: &'a Allocator, program: &mut Program<'a>) -> MinifierReturn {
        self.build(CompressionMode::Full, allocator, program)
    }

    pub fn dce(self, allocator: &'a Allocator, program: &mut Program<'a>) -> MinifierReturn {
        self.build(CompressionMode::TreeShakeOnly, allocator, program)
    }

    fn build(
        self,
        mode: CompressionMode,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
    ) -> MinifierReturn {
        let Self { options } = self;
        let MinifierOptions { mangle, mangle_properties, compress } = options;

        // Run property mangling first because compression can change which strings are property
        // keys.
        let property_mangle_cache = mangle_properties.map(|options| {
            let mut mangler = PropertyMangler::new_in(options, allocator);
            mangler.collect(program);
            mangler.assign();
            mangler.rewrite(program);
            mangler.into_cache()
        });

        let (stats, iterations) = compress
            .map(|options| {
                let semantic = SemanticBuilder::new().build(program).semantic;
                let stats = semantic.stats();
                let scoping = semantic.into_scoping();
                let compressor = Compressor::new(allocator);
                let iterations = if matches!(mode, CompressionMode::TreeShakeOnly) {
                    let options = CompressOptions {
                        target: options.target,
                        treeshake: options.treeshake,
                        ..CompressOptions::dce()
                    };
                    compressor.dead_code_elimination_with_scoping(program, scoping, options)
                } else {
                    compressor.build_with_scoping(program, scoping, options)
                };
                (Some(stats), iterations)
            })
            .unwrap_or_default();
        let (scoping, class_private_mappings) = mangle
            .map(|options| {
                let mut builder =
                    SemanticBuilder::new().with_build_nodes(true).with_class_table(true);
                if let Some(stats) = stats {
                    builder = builder.with_stats(stats);
                }
                let mut semantic = builder.build(program).semantic;
                let mut options = options;
                let aliased = annex_b_alias::alias_annex_b_block_functions(
                    &mut semantic,
                    &mut options.reserved,
                );
                if options.keep_names.function {
                    options.reserved.extend(aliased);
                    reserve_function_names(&semantic, &mut options.reserved);
                }
                reserve_with_shadowed_bindings(semantic.scoping(), &mut options.reserved);
                // A root binding that redeclares a CommonJS wrapper parameter is
                // mapped to the wrapper's `arguments`, which renaming would break.
                if program.source_type.is_commonjs() {
                    let scoping = semantic.scoping();
                    options.reserved.extend(
                        state::COMMONJS_WRAPPER_PARAMETERS
                            .into_iter()
                            .filter(|name| {
                                scoping
                                    .get_binding(scoping.root_scope_id(), (*name).into())
                                    .is_some()
                            })
                            .map(CompactStr::from),
                    );
                }
                let class_private_mappings = Mangler::default()
                    .with_options(options)
                    .build_with_semantic(&mut semantic, program);
                (semantic.into_scoping(), class_private_mappings)
            })
            .map_or((None, None), |(scoping, mappings)| (Some(scoping), Some(mappings)));
        MinifierReturn { scoping, class_private_mappings, property_mangle_cache, iterations }
    }
}

/// Reserve bindings whose resolved references cross a `with` statement scope so
/// the mangler does not rename an identifier that a `with` object may intercept.
fn reserve_with_shadowed_bindings(
    scoping: &Scoping,
    reserved: &mut rustc_hash::FxHashSet<CompactStr>,
) {
    use oxc_syntax::scope::ScopeFlags;
    if !scoping
        .scope_descendants_from_root()
        .any(|scope_id| scoping.scope_flags(scope_id).contains(ScopeFlags::With))
    {
        return;
    }
    for symbol_id in scoping.symbol_ids() {
        let decl_scope = scoping.symbol_scope_id(symbol_id);
        let crosses_with = scoping.get_resolved_references(symbol_id).any(|reference| {
            scoping
                .scope_ancestors(reference.scope_id())
                .take_while(|&scope_id| scope_id != decl_scope)
                .any(|scope_id| scoping.scope_flags(scope_id).contains(ScopeFlags::With))
        });
        if crosses_with {
            reserved.insert(CompactStr::from(scoping.symbol_name(symbol_id)));
        }
    }
}

/// Reserve function names, including inferred names from parameter defaults.
fn reserve_function_names(
    semantic: &oxc_semantic::Semantic<'_>,
    reserved: &mut rustc_hash::FxHashSet<CompactStr>,
) {
    use oxc_ast::{AstKind, ast::*};
    use oxc_syntax::symbol::SymbolId;

    fn is_anonymous_fn(expr: &Expression<'_>) -> bool {
        expr.is_anonymous_function_definition()
            && !matches!(expr.without_parentheses(), Expression::ClassExpression(_))
    }

    fn find_assign_pattern_for_symbol<'a>(
        pattern: &'a BindingPattern<'a>,
        symbol_id: SymbolId,
    ) -> Option<&'a AssignmentPattern<'a>> {
        match pattern {
            BindingPattern::BindingIdentifier(_) => None,
            BindingPattern::ObjectPattern(obj) => obj
                .properties
                .iter()
                .find_map(|p| find_assign_pattern_for_symbol(&p.value, symbol_id)),
            BindingPattern::ArrayPattern(arr) => arr
                .elements
                .iter()
                .flatten()
                .find_map(|el| find_assign_pattern_for_symbol(el, symbol_id)),
            BindingPattern::AssignmentPattern(assign) => {
                if matches!(&assign.left, BindingPattern::BindingIdentifier(id) if id.symbol_id() == symbol_id)
                {
                    return Some(assign);
                }
                find_assign_pattern_for_symbol(&assign.left, symbol_id)
            }
        }
    }

    let scoping = semantic.scoping();
    let ast_nodes = semantic.nodes();

    for symbol_id in scoping.symbol_ids() {
        let decl_node = ast_nodes.get_node(scoping.symbol_declaration(symbol_id));
        let mut is_fn_name_symbol = match decl_node.kind() {
            AstKind::Function(function) => {
                function.id.as_ref().is_some_and(|id| id.symbol_id() == symbol_id)
            }
            AstKind::VariableDeclarator(decl) => {
                if let BindingPattern::BindingIdentifier(id) = &decl.id
                    && id.symbol_id() == symbol_id
                {
                    decl.init.as_ref().is_some_and(is_anonymous_fn)
                } else {
                    find_assign_pattern_for_symbol(&decl.id, symbol_id)
                        .is_some_and(|assign| is_anonymous_fn(&assign.right))
                }
            }
            AstKind::FormalParameter(param) => {
                if let BindingPattern::BindingIdentifier(id) = &param.pattern
                    && id.symbol_id() == symbol_id
                {
                    param.initializer.as_ref().is_some_and(|init| is_anonymous_fn(init))
                } else {
                    find_assign_pattern_for_symbol(&param.pattern, symbol_id)
                        .is_some_and(|assign| is_anonymous_fn(&assign.right))
                }
            }
            _ => false,
        };
        if !is_fn_name_symbol {
            is_fn_name_symbol =
                scoping.get_resolved_reference_ids(symbol_id).iter().any(|&ref_id| {
                    let node_id = scoping.get_reference(ref_id).node_id();
                    let parent_id = ast_nodes.parent_id(node_id);
                    match ast_nodes.kind(parent_id) {
                        AstKind::AssignmentExpression(assign) => {
                            matches!(&assign.left, AssignmentTarget::AssignmentTargetIdentifier(id) if id.reference_id() == ref_id)
                                && is_anonymous_fn(&assign.right)
                        }
                        AstKind::AssignmentTargetWithDefault(assign) => {
                            matches!(&assign.binding, AssignmentTarget::AssignmentTargetIdentifier(id) if id.reference_id() == ref_id)
                                && is_anonymous_fn(&assign.init)
                        }
                        AstKind::AssignmentTargetPropertyIdentifier(ident) => {
                            ident.binding.reference_id() == ref_id
                                && ident.init.as_ref().is_some_and(is_anonymous_fn)
                        }
                        _ => false,
                    }
                });
        }
        if is_fn_name_symbol {
            reserved.insert(CompactStr::from(scoping.symbol_name(symbol_id)));
        }
    }
}
