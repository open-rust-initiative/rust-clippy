use super::PASSING_STRING_TO_C_FUNCTIONS;
use clippy_utils::diagnostics::span_lint_and_help;
use clippy_utils::ty::is_type_lang_item;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::intravisit::{walk_expr, Visitor};
use rustc_hir::{Expr, ExprKind, HirId, LangItem, Node, QPath};
use rustc_lint::LateContext;
use rustc_middle::ty;
use rustc_span::Span;

struct ParamsFinder {
    params: Vec<(HirId, Span)>,
}

impl<'tcx> Visitor<'tcx> for ParamsFinder {
    fn visit_expr(&mut self, ex: &'tcx Expr<'tcx>) {
        if let ExprKind::Path(QPath::Resolved(None, name)) = &ex.kind {
            if let Res::Local(id) = name.res {
                self.params.push((id, ex.span));
            }
        }

        walk_expr(self, ex);
    }
}

pub(super) fn check_expr<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
    let mut foreign_params: Option<Vec<(HirId, Span)>> = None;
    if let ExprKind::Call(func, params) = expr.kind {
        if let ExprKind::Path(QPath::Resolved(None, path)) = func.kind {
            if let Res::Def(DefKind::Fn, def_id) = path.res {
                let Some(def_id) = def_id.as_local() else { return };
                let func_id = cx.tcx.hir().local_def_id_to_hir_id(def_id);
                if let Node::ForeignItem(..) = cx.tcx.hir().get(func_id) {
                    let mut finder = ParamsFinder { params: Vec::new() };
                    for param in params {
                        finder.visit_expr(param);
                    }
                    foreign_params = Some(finder.params);
                }
            }
        }
    }
    let Some(params) = foreign_params else { return };

    for (param, span) in params {
        let ty = cx.typeck_results().node_type(param);
        match ty.kind() {
            ty::Ref(_, t, _) if *t.kind() == ty::Str => {
                span_lint_and_help(
                    cx,
                    PASSING_STRING_TO_C_FUNCTIONS,
                    expr.span,
                    "can't pass rust string to ffi functions!",
                    Some(span),
                    "use `Cstring` or `Cstr` instead",
                );
            },
            _ if is_type_lang_item(cx, ty, LangItem::String) => {
                span_lint_and_help(
                    cx,
                    PASSING_STRING_TO_C_FUNCTIONS,
                    expr.span,
                    "can't pass rust String to ffi functions!",
                    Some(span),
                    "use `Cstring` or `Cstr` instead",
                );
            },
            _ => (),
        }
    }
}
