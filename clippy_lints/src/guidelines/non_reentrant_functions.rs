use super::NON_REENTRANT_FUNCTIONS;
use clippy_utils::diagnostics::span_lint;
use rustc_hir::{Expr, ExprKind, QPath, Path};
use rustc_lint::LateContext;

pub(super) fn check(cx: &LateContext<'_>, expr: &Expr<'_>) {
    let msg: &str = "consider using the reentrant version of the function";

    if let ExprKind::Call(func, _) = &expr.kind {
        if is_reentrant_fn(func) {
            span_lint(cx, NON_REENTRANT_FUNCTIONS, expr.span, msg);
        }
    }
}

fn is_reentrant_fn(func: &Expr<'_>) -> bool {
    match &func.kind {
        ExprKind::Path(QPath::Resolved(None, Path { segments, .. })) => {
            if segments.len() != 2 || segments[0].ident.name != rustc_span::sym::libc {
                return false;
            }
            matches!(segments[1].ident.as_str(), "strtok" | "localtime")
        },
        _ => false,
    }
}
