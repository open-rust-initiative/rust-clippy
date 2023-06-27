use super::FALLIABLE_MEMORY_ALLOCATION;
use clippy_utils::diagnostics::{span_lint, span_lint_and_note};
use rustc_hir::def::Res;
use rustc_hir::intravisit::walk_expr;
use rustc_hir::intravisit::Visitor;
use rustc_hir::{Expr, ExprKind, HirId, Node, QPath, BinOpKind};
use rustc_lint::LateContext;
use rustc_span::symbol::Ident;
use rustc_span::Span;

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
    id: Option<(HirId, Span)>,
    max_find: bool,
    null_find: bool,
}

impl<'tcx> Visitor<'tcx> for MaxNullFinder {
    fn visit_expr(&mut self, ex: &'tcx Expr<'tcx>) {
        if let Some((size, _)) = self.id && !self.max_find && let ExprKind::Binary(op, ex1, _) = ex.kind {
            if let ExprKind::Path(QPath::Resolved(None, name)) = ex1.kind {
                if let Res::Local(id) = name.res && id == size {
                    if matches!(op.node, BinOpKind::Le | BinOpKind::Lt | BinOpKind::Eq | BinOpKind::Ge | BinOpKind::Gt) {
                        self.max_find = true;
                    }
                }
            }
        }

        walk_expr(self, ex);
    }

    fn visit_ident(&mut self, ident: Ident) {
        if !self.max_find
            && (ident.as_str().contains("MAX") || ident.as_str().contains("max") || ident.as_str().contains("min"))
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
        let mut finder = NameFinder {
            find: false,
            ptr: false,
        };
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
        let mut finder = MaxNullFinder {
            id: size,
            max_find: false,
            null_find: false,
        };
        for (_, node) in cx.tcx.hir().parent_iter(expr.hir_id) {
            if let Node::Block(block) = node {
                finder.visit_block(block);
            }
        }
        if let Some((_size, span)) = size {
            if !finder.max_find {
                span_lint_and_note(
                    cx,
                    FALLIABLE_MEMORY_ALLOCATION,
                    expr.span,
                    "must verify size when allocate memories!",
                    Some(span),
                    "unverified allocate size used here",
                );
            }
        }
        if ptr {
            if !finder.null_find {
                span_lint(
                    cx,
                    FALLIABLE_MEMORY_ALLOCATION,
                    expr.span,
                    "must verify null pointer after allocating memories!",
                );
            }
        }
    }
}
