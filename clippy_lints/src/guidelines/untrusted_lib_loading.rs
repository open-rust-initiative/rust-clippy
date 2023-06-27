use rustc_hir::def::{DefKind, Res};
use rustc_hir::intravisit::{walk_expr, Visitor};
use rustc_hir::{ExprKind, HirId, Node, QPath, Expr, PathSegment, TyKind};
use rustc_span::Span;
use rustc_lint::LateContext;

struct IOFinder {
    find_io: Option<Span>,
    in_func: Option<Span>,
}

impl<'tcx> Visitor<'tcx> for IOFinder {
    fn visit_expr(&mut self, ex: &'tcx Expr<'tcx>) {
        if self.find_io.is_none() && let ExprKind::Call(..) = &ex.kind {
            self.in_func = Some(ex.span);
            walk_expr(self, ex);
            self.in_func = None;
        } else {
            walk_expr(self, ex);
        }
    }

    fn visit_path_segment(&mut self, path_segment: &'tcx PathSegment<'tcx>) {
        if let Some(span) = self.in_func {
            if path_segment.ident.as_str().starts_with("open") && self.find_io.is_none() {
                self.find_io = Some(span);
            }
            return;
        }
    }
}

struct LibNameFinder {
    libname: Option<HirId>,
}

impl<'tcx> Visitor<'tcx> for LibNameFinder {
    fn visit_expr(&mut self, ex: &'tcx Expr<'tcx>) {
        if self.libname.is_none() && let ExprKind::Path(QPath::Resolved(None, name)) = &ex.kind {
            if let Res::Local(id) = name.res {
                self.libname = Some(id);
            }
        }

        walk_expr(self, ex);
    }
}

// TODO: Adjust the parameters as necessary
pub(crate) fn check_expr<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
    let mut libname = None;
    let mut loading: Option<Span> = None;
    if let ExprKind::Call(func, params) = &expr.kind {
        if let ExprKind::Path(QPath::TypeRelative(ty, segment)) = func.kind {
            if let TyKind::Path(qpath) = &ty.kind {
                match qpath {
                    QPath::Resolved(None, p) => {
                        if let Res::Def(DefKind::Struct, def_id) = p.res  {
                            if cx.tcx.crate_name(def_id.krate).as_str() == "libloading" {
                                if p.segments.last().unwrap().ident.as_str() == "Library" {
                                    loading = Some(expr.span);
                                    if segment.ident.as_str() == "new" && params.len() > 0 {
                                        let mut libname_finder = LibNameFinder { libname: None };
                                        libname_finder.visit_expr(&params[0]);
                                        if let Some(id) = libname_finder.libname {
                                            libname = Some(id);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {},
                }
            }
        }
    }
    let libname = match libname {
        Some(id) => id,
        None => return,
    };

    for (_, node) in cx.tcx.hir().parent_iter(libname) {
        if let Node::Block(block) = node {
            let mut finder = IOFinder { find_io: None, in_func: None };
            finder.visit_block(block);

            if let Some(span) = finder.find_io {
                let mut err = cx.tcx.sess.diagnostic().struct_span_err(span, "can't read outer files when loading the dynamic libraries!");
                err.span_note(loading.unwrap(), "loading dynamic library here");
                err.emit();
                break;
            }
        }
    }
}
