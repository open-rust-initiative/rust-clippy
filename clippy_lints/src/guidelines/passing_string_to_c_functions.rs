use rustc_hir::{ExprKind, HirId, LangItem, Node, QPath, Expr};
use rustc_hir::def::{DefKind, Res};
use rustc_hir::intravisit::{Visitor, walk_expr};
use rustc_middle::ty;
use rustc_span::Span;
use clippy_utils::ty::is_type_lang_item;
use rustc_lint::LateContext;

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
                let def_id = match def_id.as_local() {
                    Some(id) => id,
                    None => return,
                };
                let func_id = cx.tcx.hir().local_def_id_to_hir_id(def_id);
                if let Node::ForeignItem(..) = cx.tcx.hir().get(func_id) {
                    let mut finder = ParamsFinder{ params: Vec::new() };
                    for param in params {
                        finder.visit_expr(param);
                    }
                    foreign_params = Some(finder.params);
                }
            }
        }
    }
    let params = match foreign_params {
        Some(params) => params,
        None => return,
    };

    for (param, span) in params.into_iter() {
        let ty = cx.typeck_results().node_type(param);
        match ty.kind() {
            ty::Ref(_, t, _) if *t.kind() == ty::Str => {
                let mut err = cx.tcx.sess.struct_span_err(expr.span, "can't pass rust string to ffi functions!");
                err.span_note(span, "local rust string used here");
                err.help("use `Cstring` or `Cstr` instead");
                err.emit();
            }
            _ if is_type_lang_item(cx, ty, LangItem::String) => {
                let mut err = cx.tcx.sess.struct_span_err(expr.span, "can't pass rust String to ffi functions!");
                err.span_note(span, "local rust String defined here");
                err.help("use `Cstring` or `Cstr` instead");
                err.emit();
            }
            _ => (),
        }
    }
}
