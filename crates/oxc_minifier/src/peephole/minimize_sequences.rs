use oxc_allocator::ArenaVec;
use oxc_ast::ast::{Expression, SequenceExpression};
use oxc_span::Span;

use crate::TraverseCtx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// Joins two expressions into a sequence, preserving evaluation order.
    pub fn join_sequence(
        span: Span,
        a: Expression<'a>,
        b: Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        if let Expression::SequenceExpression(mut sequence_expr) = a {
            // `(a, b); c`
            if let Expression::SequenceExpression(b_seq) = b {
                sequence_expr.expressions.extend(b_seq.unbox().expressions);
            } else {
                sequence_expr.expressions.push(b);
            }
            sequence_expr.span = span;
            return Expression::SequenceExpression(sequence_expr);
        }
        let exprs = if let Expression::SequenceExpression(sequence_expr) = b {
            // `a; (b, c)`
            let b_exprs = sequence_expr.unbox().expressions;
            let mut exprs = ArenaVec::with_capacity_in((b_exprs.len() + 1).max(4), ctx);
            exprs.push(a);
            exprs.extend(b_exprs);
            exprs
        } else {
            // `a; b`
            let mut exprs = ArenaVec::with_capacity_in(4, ctx);
            exprs.push(a);
            exprs.push(b);
            exprs
        };
        Expression::new_sequence_expression(span, exprs, ctx)
    }

    /// Returns the final expression, descending through non-empty trailing sequences.
    /// For `(a, (b, c))`, returns `c`. An empty outer sequence returns [`None`];
    /// an empty trailing sequence is returned as-is.
    pub fn last_expression_in_sequence<'s>(
        seq: &'s SequenceExpression<'a>,
    ) -> Option<&'s Expression<'a>> {
        let mut last = seq.expressions.last()?;
        while let Expression::SequenceExpression(inner) = last {
            let Some(next) = inner.expressions.last() else { break };
            last = next;
        }
        Some(last)
    }

    /// Removes and returns the final expression, descending through trailing sequences
    /// and discarding any that become empty.
    /// For `(a, (b, c))`, returns `c` and leaves `(a, (b))`.
    /// Returns [`None`] if the outer sequence or a trailing nested sequence is already empty.
    pub fn pop_expression_from_sequence(
        seq: &mut SequenceExpression<'a>,
    ) -> Option<Expression<'a>> {
        if let Some(Expression::SequenceExpression(inner)) = seq.expressions.last_mut() {
            let last = Self::pop_expression_from_sequence(inner)?;
            if inner.expressions.is_empty() {
                seq.expressions.pop();
            }
            return Some(last);
        }

        seq.expressions.pop()
    }
}
