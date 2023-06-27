use rustc_hir::def::Res;
use rustc_hir::intravisit::Visitor;
use rustc_hir::{ExprKind, HirId, Node, QPath, Expr};
use rustc_span::Span;
use rustc_span::symbol::Ident;
use rustc_lint::LateContext;

struct NameFinder {
    find: bool,
    ptr: bool,
}

impl<'tcx> Visitor<'tcx> for NameFinder {
    fn visit_ident(&mut self, ident: Ident) {
        if !self.find {
            if ident.as_str().starts_with("with_capacity") {
                self.find = true;
            } else if ident.as_str().starts_with("malloc") || ident.as_str().starts_with("allocate") {
                self.find = true;
                self.ptr = true;
            }
        }
    }
}

struct MaxNullFinder {
    max_find: bool,
    null_find: bool,
}

impl<'tcx> Visitor<'tcx> for MaxNullFinder {
    fn visit_ident(&mut self, ident: Ident) {
        if !self.max_find
            && (ident.as_str().contains("MAX")
            || ident.as_str().contains("max")
            || ident.as_str().contains("min"))
        {
            self.max_find = true;
        }
        if !self.null_find && ident.as_str().contains("is_null") {
            self.null_find = true;
        }
    }
}

// TODO: Adjust the parameters as necessary
pub(super) fn check_expr<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
    let mut size: Option<(HirId, Span)> = None;
    let mut ptr: bool = false;
    if let ExprKind::Call(func, params) = expr.kind {
        let mut finder = NameFinder { find: false, ptr: false };
        finder.visit_expr(func);
        if finder.find {
            if params.len() == 1 && let ExprKind::Path(QPath::Resolved(None, name)) = params[0].kind {
                if let Res::Local(id) = name.res {
                    size = Some((id, name.span));
                }
            }
            if finder.ptr {
                ptr = true;
            }
        }
    }

    if size.is_some() || ptr {
        let mut finder = MaxNullFinder { max_find: false, null_find: false };
        for (_, node) in cx.tcx.hir().parent_iter(expr.hir_id) {
            if let Node::Block(block) = node {
                finder.visit_block(block);
            }
        }
        if let Some((_size, span)) = size {
            if !finder.max_find {
                let mut err = cx
                    .tcx
                    .sess
                    .diagnostic()
                    .struct_span_err(expr.span, "must verify size when allocate memories!");
                err.span_note(span, "unverified allocate size used here");
                err.emit();
            }
        }
        if ptr {
            if !finder.null_find {
                let mut err = cx.tcx.sess.diagnostic().struct_span_err(
                    expr.span,
                    "must verify null pointer after allocating memories!",
                );
                err.emit();
            }
        }
    }
}
