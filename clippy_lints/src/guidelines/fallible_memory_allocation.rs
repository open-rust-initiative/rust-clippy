use super::FALLIBLE_MEMORY_ALLOCATION;
use clippy_utils::diagnostics::{span_lint, span_lint_and_note};
use clippy_utils::source::snippet_opt;
use rustc_hir::def::Res;
use rustc_hir::def_id::DefId;
use rustc_hir::intravisit::Visitor;
use rustc_hir::intravisit::{walk_expr, walk_stmt};
use rustc_hir::{Expr, ExprKind, HirId, Local, Node, Path, PathSegment, QPath, Stmt, StmtKind};
use rustc_lint::LateContext;
use rustc_span::symbol::Ident;
use rustc_span::Span;

enum PtrStatus {
    Verified,
    Unverified,
    NotPtr,
}

struct VerifierFinder<'a> {
    id: HirId,
    size_verified: bool,
    ptr_verified: PtrStatus,
    alloc_size_check_fns: &'a [String],
}

impl<'tcx, 'a> Visitor<'tcx> for VerifierFinder<'a> {
    fn visit_expr(&mut self, ex: &'tcx Expr<'tcx>) {
        match &ex.kind {
            // e.g. `left < right` or `left >= right` etc.
            ExprKind::Binary(op, left, right) => {
                let is_compared = |expr: &Expr<'_>| -> bool {
                    if let ExprKind::Path(QPath::Resolved(None, name)) = expr.kind {
                        if let Res::Local(id) = name.res && id == self.id && op.node.is_comparison() {
                            return true;
                        }
                    }
                    false
                };

                if is_compared(left) || is_compared(right) {
                    self.size_verified = true;
                }
            },
            ExprKind::Lit(..) => {
                // don't take hard coded literal into account
                self.size_verified = true;
                return;
            },
            _ => (),
        }
        walk_expr(self, ex);
    }
    fn visit_stmt(&mut self, stmt: &'tcx Stmt<'tcx>) {
        if let StmtKind::Local(Local {
            pat, init: Some(init), ..
        }) = stmt.kind
        {
            if pat.hir_id != self.id {
                return;
            }
            if let Some(true) = expr_is_checker_call(init, self.alloc_size_check_fns) {
                self.size_verified = true;
            }
        }
        walk_stmt(self, stmt);
    }
    fn visit_ident(&mut self, ident: Ident) {
        if let PtrStatus::Unverified = self.ptr_verified {
            if ident.as_str().contains("is_null") {
                self.ptr_verified = PtrStatus::Verified;
            }
        }
    }
}

pub(super) fn check_expr<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx Expr<'tcx>,
    params: &'tcx [Expr<'tcx>],
    func_did: DefId,
    alloc_size_check_fns: &[String],
) {
    let mut path_to_check: Option<(HirId, Span)> = None;
    let mut ptr_status = PtrStatus::Unverified;
    let mut maybe_size_param: Option<&Expr<'_>> = None;

    if !cx.tcx.fn_sig(func_did).skip_binder().output().is_unsafe_ptr() {
        ptr_status = PtrStatus::NotPtr;
    }

    if let [param] = params {
        maybe_size_param = Some(param);
    } else {
        for param in params {
            if matches!(snippet_opt(cx, param.span), Some(s) if s.contains("size")) {
                maybe_size_param = Some(param);
                break;
            }
        }
    }

    let Some(size_param) = maybe_size_param else { return };
    if let Some(id_and_span) = path_hir_id_and_span(size_param) {
        path_to_check = Some(id_and_span);
    } else if let Some(false) = expr_is_checker_call(size_param, alloc_size_check_fns) {
        warn_unverified_size(cx, expr.span, size_param.span);
    } else {
        return;
    }

    if let Some((hid, span)) = path_to_check {
        let mut finder = VerifierFinder {
            id: hid,
            size_verified: false,
            ptr_verified: ptr_status,
            alloc_size_check_fns,
        };
        for (_, node) in cx.tcx.hir().parent_iter(expr.hir_id) {
            if let Node::Block(block) = node {
                finder.visit_block(block);
            }
        }

        if !finder.size_verified {
            warn_unverified_size(cx, expr.span, span);
        }
        if let PtrStatus::Unverified = finder.ptr_verified {
            span_lint(
                cx,
                FALLIBLE_MEMORY_ALLOCATION,
                expr.span,
                "allocating memory without checking if the result pointer is null",
            );
        }
    }
}

fn expr_is_checker_call(expr: &Expr<'_>, checker_fns: &[String]) -> Option<bool> {
    match &expr.kind {
        ExprKind::Call(
            Expr {
                kind:
                    ExprKind::Path(QPath::Resolved(
                        None,
                        Path {
                            segments: [PathSegment { ident, .. }, ..],
                            ..
                        },
                    )),
                ..
            },
            _,
        )
        | ExprKind::MethodCall(PathSegment { ident, .. }, ..) => Some(is_checker_fn(ident, checker_fns)),
        _ => None,
    }
}

fn is_checker_fn(ident: &Ident, fns: &[String]) -> bool {
    for fn_name in fns {
        if ident.as_str().contains(fn_name) {
            return true;
        }
    }
    false
}

fn warn_unverified_size(cx: &LateContext<'_>, span: Span, hint_span: Span) {
    span_lint_and_note(
        cx,
        FALLIBLE_MEMORY_ALLOCATION,
        span,
        "allocating memory without verifying the size",
        Some(hint_span),
        "unverified size used here",
    );
}

/// Get [`HirId`] and [`Span`] of a node if given expr could be resolved to [`Res::Local`].
fn path_hir_id_and_span(maybe_path: &Expr<'_>) -> Option<(HirId, Span)> {
    if let ExprKind::Path(QPath::Resolved(
        None,
        Path {
            res: Res::Local(hid),
            span,
            ..
        },
    )) = &maybe_path.kind
    {
        Some((*hid, *span))
    } else {
        None
    }
}
