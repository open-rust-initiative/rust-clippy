use super::UNTRUSTED_LIB_LOADING;
use clippy_utils::diagnostics::span_lint_and_note;
use clippy_utils::visitors::for_each_expr;
use clippy_utils::{fn_def_id, path_to_local};
use core::ops::ControlFlow;
use if_chain::if_chain;
use rustc_hir::def_id::DefIdSet;
use rustc_hir::intravisit::{walk_stmt, Visitor};
use rustc_hir::{Expr, ExprKind, HirId, Local, Node, Stmt, StmtKind};
use rustc_lint::LateContext;
use rustc_span::Span;

#[rustfmt::skip]
pub(super) const IO_FUNCTIONS: &[&str] = &[
    // Native io functions
    "std::io::Read::read",
    "std::io::Read::read_to_end",
    "std::io::Read::read_to_string",
    "std::io::Read::read_exact",
    "std::io::Stdin::read_line",
    "std::fs::read",
    "std::fs::read_to_string",
    // standard input
    "gets", "getchar",
    // formatted input functions
    "scanf", "wscanf", "vscanf", "vwscanf", "fscanf", "fwscanf", "vfscanf", "vfwscanf",
    "sscanf", "swscanf", "vsscanf", "vswscanf",
];
pub(super) const LOADING_FNS: &[&str] = &["libloading::Library::new", "dlopen"];

struct IOFinder<'a, 'tcx> {
    io_functions: &'a DefIdSet,
    /// Stop looking when this span is reached,
    /// this is to prevent the `expr`s in the lib loading calls being revisited.
    stop_span: Span,
    call_span: Option<Span>,
    in_io_fn: Option<Span>,
    cx: &'a LateContext<'tcx>,
    hid: HirId,
}

impl<'a, 'tcx> Visitor<'tcx> for IOFinder<'a, 'tcx> {
    // Exprs are handled manually in `visit_stmt` function.
    fn visit_expr(&mut self, _ex: &'tcx Expr<'tcx>) {}

    fn visit_stmt(&mut self, stmt: &'tcx Stmt<'tcx>) {
        if stmt.span >= self.stop_span {
            return;
        }

        match stmt.kind {
            StmtKind::Local(Local {
                pat, init: Some(init), ..
            }) => {
                if pat.hir_id == self.hid {
                    let found = for_each_expr(*init, |ex| {
                        if matches!(fn_def_id(self.cx, ex), Some(fn_did) if self.io_functions.contains(&fn_did)) {
                            self.call_span = Some(ex.span);
                            ControlFlow::Break(true)
                        } else {
                            ControlFlow::Continue(())
                        }
                    });
                    if found == Some(true) {
                        return;
                    }
                }
            },
            StmtKind::Expr(expr) | StmtKind::Semi(expr) => {
                for_each_expr(expr, |ex| {
                    if self.in_io_fn.is_none() {
                        if matches!(fn_def_id(self.cx, ex), Some(fn_did) if self.io_functions.contains(&fn_did)) {
                            self.in_io_fn = Some(ex.span);
                        }
                    } else if matches!(path_to_local(ex), Some(arg_hid) if arg_hid == self.hid) {
                        self.call_span = self.in_io_fn;
                        return ControlFlow::Break(());
                    }
                    ControlFlow::Continue(())
                });
            },
            _ => (),
        };

        walk_stmt(self, stmt);
    }
}

fn get_resolved_path_id(expr: &Expr<'_>) -> ControlFlow<HirId, ()> {
    if let Some(id) = path_to_local(expr) {
        ControlFlow::Break(id)
    } else {
        ControlFlow::Continue(())
    }
}

pub(crate) fn check<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx Expr<'tcx>,
    io_functions: &DefIdSet,
    loader_ids: &DefIdSet,
) {
    if_chain! {
        if let ExprKind::Call(_, [param, ..]) = &expr.kind;
        if let Some(fn_did) = fn_def_id(cx, expr);
        if loader_ids.contains(&fn_did);
        if let Some(hid) = for_each_expr(param, get_resolved_path_id);
        then {
            for (_, node) in cx.tcx.hir().parent_iter(hid) {
                if let Node::Block(block) = node {
                    let mut finder = IOFinder {
                        io_functions,
                        stop_span: expr.span,
                        call_span: None,
                        in_io_fn: None,
                        cx,
                        hid,
                    };
                    finder.visit_block(block);

                    if let Some(span) = finder.call_span {
                        span_lint_and_note(
                            cx,
                            UNTRUSTED_LIB_LOADING,
                            expr.span,
                            "loading dynamic library from untrusted source",
                            Some(span),
                            "untrusted IO function called here",
                        );
                        break;
                    }
                }
            }
        }
    }
}
