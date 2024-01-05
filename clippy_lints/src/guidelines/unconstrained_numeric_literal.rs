use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::is_lint_allowed;
use clippy_utils::source::snippet_opt;
use rustc_errors::{Applicability, MultiSpan};
use rustc_hir::intravisit::{walk_expr, Visitor};
use rustc_hir::{Expr, ExprKind, Local, TyKind};
use rustc_lint::{LateContext, LintContext};
use rustc_middle::lint::in_external_macro;
use rustc_span::Span;

use super::UNCONSTRAINED_NUMERIC_LITERAL;

pub(super) fn check_local<'tcx>(cx: &LateContext<'tcx>, local: &'tcx Local<'tcx>) {
    if !is_lint_allowed(cx, UNCONSTRAINED_NUMERIC_LITERAL, local.hir_id)
        && let Some(init) = local.init
        && local_has_implicit_ty(local)
    {
        let mut visitor = LitVisitor::new(cx);
        visitor.visit_expr(init);

        // The type could be wildcard (`_`), therefore we need to include its span for suggestion.
        let span = if let Some(ty) = local.ty {
            local.pat.span.to(ty.span)
        } else {
            local.pat.span
        };

        if !visitor.unconstrained_lit_spans.is_empty() {
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
                        MultiSpan::from_spans(visitor.unconstrained_lit_spans),
                        "unconstrained numeric literals defined here",
                    );
                }
            );
        }
    }
}

fn local_has_implicit_ty(local: &Local<'_>) -> bool {
    match local.ty {
        Some(ty) if matches!(ty.kind, TyKind::Infer) => true,
        None => true,
        _ => false,
    }
}

struct LitVisitor<'hir, 'tcx> {
    cx: &'hir LateContext<'tcx>,
    unconstrained_lit_spans: Vec<Span>,
}

impl<'hir, 'tcx> LitVisitor<'hir, 'tcx> {
    fn new(cx: &'hir LateContext<'tcx>) -> Self {
        Self {
            cx,
            unconstrained_lit_spans: vec![],
        }
    }
}

impl<'hir, 'tcx> Visitor<'hir> for LitVisitor<'hir, 'tcx> {
    fn visit_expr(&mut self, ex: &'hir Expr<'hir>) {
        if let ExprKind::Lit(lit) = ex.kind {
            if lit.node.is_numeric() && lit.node.is_unsuffixed() && !in_external_macro(self.cx.sess(), lit.span) {
                self.unconstrained_lit_spans.push(lit.span);
            }
        } else {
            walk_expr(self, ex);
        }
    }

    // Don't visit local in this visitor, `Local`s are handled in `check_local` call.
    fn visit_local(&mut self, _: &'hir Local<'hir>) {}
}

fn ty_suggestion(cx: &LateContext<'_>, init: &Expr<'_>) -> String {
    let ty = cx.typeck_results().expr_ty(init);
    ty.to_string()
}
