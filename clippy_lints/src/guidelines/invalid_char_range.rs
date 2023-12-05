use clippy_utils::consts::{constant, Constant};
use clippy_utils::diagnostics::span_lint_and_note;
use clippy_utils::{def_path_def_ids, is_lint_allowed};
use rustc_hir::{Expr, ExprKind};
use rustc_lint::LateContext;
use rustc_span::def_id::DefId;

use super::INVALID_CHAR_RANGE;

pub(super) fn check_call(cx: &LateContext<'_>, expr: &Expr<'_>, params: &[Expr<'_>], def_id: DefId) {
    if is_lint_allowed(cx, INVALID_CHAR_RANGE, expr.hir_id) || !is_of_from_methods(cx, def_id) {
        return;
    }

    let [param] = params else { return };
    let param = peel_casts(param);
    let tyck_res = cx.typeck_results();
    // Skip checks when input cannot be evaluated at run time.
    if let Some(Constant::Int(n)) = constant(cx, tyck_res, param) {
        if (n > 0xD7FF && n < 0xE000) || n > 0x0010_FFFF {
            span_lint_and_note(
                cx,
                INVALID_CHAR_RANGE,
                expr.span,
                "converting to char with out-of-range integer",
                Some(param.span),
                "this number should be within the range of [0, 0xD7FF] or [0xE000, 0x10FFFF]",
            );
        }
    }
}

fn peel_casts<'tcx>(expr: &'tcx Expr<'tcx>) -> &'tcx Expr<'tcx> {
    if let ExprKind::Cast(inner, _) = expr.kind {
        peel_casts(inner)
    } else {
        expr
    }
}

/// Determine if a call's id is either `char::from_u32` or `char::from_u32_unchecked`
fn is_of_from_methods(cx: &LateContext<'_>, def_id: DefId) -> bool {
    def_path_def_ids(cx, &["char", "from_u32"])
        .chain(def_path_def_ids(cx, &["char", "from_u32_unchecked"]))
        .any(|id| id == def_id)
}
