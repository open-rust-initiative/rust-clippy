use super::NON_REENTRANT_FUNCTIONS;
use clippy_utils::diagnostics::span_lint_and_help;
use clippy_utils::fn_def_id;
use rustc_hir::def_id::DefIdSet;
use rustc_hir::Expr;
use rustc_lint::LateContext;

pub(super) fn check(cx: &LateContext<'_>, expr: &Expr<'_>, blacklist_ids: &DefIdSet) {
    if let Some(did) = fn_def_id(cx, expr) {
        if blacklist_ids.contains(&did) {
            span_lint_and_help(
                cx,
                NON_REENTRANT_FUNCTIONS,
                expr.span,
                "use of non-reentrant function",
                None,
                "consider using its reentrant counterpart",
            );
        }
    }
}
