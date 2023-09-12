use std::ops::ControlFlow;

use clippy_utils::consts::{constant, Constant};
use clippy_utils::diagnostics::span_lint_and_note;
use clippy_utils::visitors::for_each_expr;
use clippy_utils::{
    is_integer_literal, is_lint_allowed, is_path_diagnostic_item, path_res, peel_hir_expr_refs, peel_hir_expr_while,
};
use rustc_hir::def::Res;
use rustc_hir::hir_id::HirId;
use rustc_hir::{Block, BorrowKind, Expr, ExprKind, Local, Node, TyKind, UnOp};
use rustc_lint::LateContext;
use rustc_span::symbol::sym;
use rustc_span::Span;

use if_chain::if_chain;

use super::NULL_PTR_DEREFERENCE;

/// Search for null pointer assigning, e.g. `x = std::ptr::null()`
pub(super) fn check_assign<'tcx, 'a: 'tcx>(cx: &LateContext<'tcx>, expr: &'a Expr<'tcx>) {
    if_chain! {
        if !is_lint_allowed(cx, NULL_PTR_DEREFERENCE, expr.hir_id);
        if !expr.span.from_expansion();
        if let ExprKind::Assign(assign_to, assigning, _) = expr.kind;
        if let Res::Local(hir_id) = path_res(cx, assign_to);
        if let Some(deref_span) = get_null_ptr_deref_span(cx, hir_id, assigning);
        then {
            lint(cx, expr.span, deref_span);
        }
    }
}

pub(super) fn check_local<'tcx, 'a: 'tcx>(cx: &LateContext<'tcx>, local: &'a Local<'tcx>) {
    if_chain! {
        if !is_lint_allowed(cx, NULL_PTR_DEREFERENCE, local.hir_id);
        if !local.span.from_expansion();
        if let Local { pat, init: Some(init_expr), .. } = local;
        if let Some(deref_span) = get_null_ptr_deref_span(cx, pat.hir_id, init_expr);
        then {
            lint(cx, local.span, deref_span);
        }
    }
}

fn get_null_ptr_deref_span<'tcx, 'a: 'tcx>(
    cx: &LateContext<'tcx>,
    ptr_var_id: HirId,
    init: &'a Expr<'tcx>,
) -> Option<Span> {
    if expr_might_be_null_ptr(cx, init) {
        // Search within the current block that this initialize/assignment happend in place
        // FIXME: this will have FN if the pointer was declared globally, i.e. `const PTR: *mut i8 = 0 as
        // *mut _;`, which should be fine at the moment,
        // since its hard to detect dereferecing on a global null pointer without the help of mir.
        if let Some(parent_blk) = get_parent_block(cx, init) {
            // This will produce:
            // { let ptr = std::ptr::null(); do_something(); }
            //       ----------------------: init
            // -----------------------------------------------: parent_blk
            //                              ------------------: search_span
            let search_span = parent_blk.span.with_lo(init.span.hi());

            let mut assume_is_null = true;
            return for_each_expr(parent_blk, |ex| {
                // Skip span that are before the initialization
                if !search_span.contains(ex.span) {
                    return ControlFlow::Continue(());
                }
                match ex.kind {
                    // check for `ptr = non_null()` and `*ptr = some_value()`
                    ExprKind::Assign(assign_to, assigning, _) => {
                        let peeled = peel_hir_expr_unary_and_refs(assign_to);
                        if path_with_desired_id(cx, peeled, ptr_var_id) && !expr_might_be_null_ptr(cx, assigning) {
                            assume_is_null = false;
                        }
                    },
                    ExprKind::Unary(UnOp::Deref, expr) => {
                        if path_with_desired_id(cx, peel_hir_expr_refs(expr).0, ptr_var_id) && assume_is_null {
                            return ControlFlow::Break(ex.span);
                        }
                    },
                    _ => {
                        if path_with_desired_id(cx, peel_hir_expr_refs(ex).0, ptr_var_id) {
                            assume_is_null = false;
                        }
                    },
                }
                ControlFlow::Continue(())
            });
        }
    }
    None
}

fn peel_hir_expr_unary_and_refs<'a>(expr: &'a Expr<'a>) -> &'a Expr<'a> {
    peel_hir_expr_while(expr, |e| match e.kind {
        ExprKind::AddrOf(BorrowKind::Ref, _, e) | ExprKind::Unary(_, e) => Some(e),
        _ => None,
    })
}

fn expr_might_be_null_ptr(cx: &LateContext<'_>, expr: &Expr<'_>) -> bool {
    match expr.kind {
        ExprKind::Path(_) if matches!(constant(cx, cx.typeck_results(), expr), Some((Constant::RawPtr(0), _))) => true,
        ExprKind::Cast(inner_expr, cast_ty)
            if is_integer_literal(peel_casts(inner_expr), 0) && matches!(cast_ty.kind, TyKind::Ptr(_)) =>
        {
            true
        },
        ExprKind::Call(call, [])
            if is_path_diagnostic_item(cx, call, sym::ptr_null)
                || is_path_diagnostic_item(cx, call, sym::ptr_null_mut) =>
        {
            true
        },
        _ => false,
    }
}

/// Peels all casts and return the inner most (non-cast) expression.
///
/// i.e.
///
/// ```
/// some_expr as *mut i8 as *mut i16 as *mut i32 as *mut i64
/// ```
///
/// Will return expression for `some_expr`.
fn peel_casts<'a, 'tcx>(maybe_cast_expr: &'a Expr<'tcx>) -> &'a Expr<'tcx> {
    if let ExprKind::Cast(expr, _) = maybe_cast_expr.kind {
        peel_casts(expr)
    } else {
        maybe_cast_expr
    }
}

fn get_parent_block<'tcx, 'a>(cx: &LateContext<'tcx>, expr: &'a Expr<'tcx>) -> Option<&'a Block<'tcx>> {
    for (_, node) in cx.tcx.hir().parent_iter(expr.hir_id) {
        if let Node::Block(block) = node {
            return Some(block);
        }
    }
    None
}

fn path_with_desired_id(cx: &LateContext<'_>, maybe_path: &Expr<'_>, id: HirId) -> bool {
    if let Res::Local(hid) = path_res(cx, maybe_path) && hid == id {
        return true;
    }
    false
}

fn lint(cx: &LateContext<'_>, decl_span: Span, deref_span: Span) {
    span_lint_and_note(
        cx,
        NULL_PTR_DEREFERENCE,
        decl_span,
        "dereferencing null pointer",
        Some(deref_span),
        "first dereference occurred here",
    );
}
