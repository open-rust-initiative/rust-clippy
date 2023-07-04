use super::PASSING_STRING_TO_C_FUNCTIONS;
use clippy_utils::diagnostics::span_lint_and_help;
use clippy_utils::visitors::for_each_expr;
use core::ops::ControlFlow;
use if_chain::if_chain;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::{Expr, ExprKind, LangItem, Path, QPath};
use rustc_lint::LateContext;
use rustc_middle::ty;

pub(super) fn check_expr<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
    if_chain! {
        if let ExprKind::Call(func, params) = expr.kind;
        if let ExprKind::Path(QPath::Resolved(None, path)) = func.kind;
        if let Res::Def(DefKind::Fn, def_id) = path.res;
        if cx.tcx.is_foreign_item(def_id);
        then {
            for param in params {
                let str_or_string: Option<&Expr<'_>> = for_each_expr(param, |e| {
                    let ExprKind::Path(QPath::Resolved(None, Path { res: Res::Local(..), .. })) = e.kind else {
                        return ControlFlow::Continue(())
                    };
                    let ty = cx.typeck_results().node_type(e.hir_id);
                    match ty.kind() {
                        ty::Ref(_, t, _) if *t.kind() == ty::Str => ControlFlow::Break(e),
                        ty::Adt(adt, _) if cx.tcx.lang_items().get(LangItem::String) == Some(adt.did())
                            => ControlFlow::Break(e),
                        _ => ControlFlow::Continue(())
                    }
                });

                if let Some(e) = str_or_string {
                    span_lint_and_help(
                        cx,
                        PASSING_STRING_TO_C_FUNCTIONS,
                        expr.span,
                        "passing native strings to external functions",
                        Some(e.span),
                        "use `CString` or `CStr` instead",
                    );
                }
            }
        }
    }
}
