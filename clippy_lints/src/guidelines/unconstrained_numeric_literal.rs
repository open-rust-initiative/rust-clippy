use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::is_lint_allowed;
use clippy_utils::source::snippet_opt;
use rustc_errors::Applicability;
use rustc_hir::{Expr, ExprKind, Local, TyKind};
use rustc_lint::{LateContext, LintContext};
use rustc_middle::lint::in_external_macro;

use super::UNCONSTRAINED_NUMERIC_LITERAL;

pub(super) fn check_local<'tcx>(cx: &LateContext<'tcx>, local: &'tcx Local<'tcx>) {
    if !is_lint_allowed(cx, UNCONSTRAINED_NUMERIC_LITERAL, local.hir_id)
        && let Some(init) = local.init
        && let ExprKind::Lit(lit) = init.kind
        && !in_external_macro(cx.sess(), lit.span)
        && (lit.node.is_numeric() && lit.node.is_unsuffixed())
        && local_has_implicit_ty(local)
    {
        // The type could be wildcard (`_`), therefore we need to include its span for suggestion.
        let span = if let Some(ty) = local.ty {
            local.pat.span.to(ty.span)
        } else {
            local.pat.span
        };

        span_lint_and_then(
            cx,
            UNCONSTRAINED_NUMERIC_LITERAL,
            span,
            "type of this numeric variable is unconstrained",
            |diag| {
                let sugg = format!(
                    "{}: {}",
                    snippet_opt(cx, local.pat.span).unwrap_or("_".to_string()),
                    ty_suggestion(cx, init),
                );
                diag.span_suggestion(
                    span,
                    "either add suffix to above numeric literal(s) or label the type explicitly",
                    sugg,
                    Applicability::MachineApplicable
                );
                diag.span_note(
                    lit.span,
                    "unconstrained numeric literals defined here",
                );
            }
        );
    }
}

fn local_has_implicit_ty(local: &Local<'_>) -> bool {
    match local.ty {
        Some(ty) if matches!(ty.kind, TyKind::Infer) => true,
        None => true,
        _ => false,
    }
}

fn ty_suggestion(cx: &LateContext<'_>, init: &Expr<'_>) -> String {
    let ty = cx.typeck_results().expr_ty(init);
    ty.to_string()
}
