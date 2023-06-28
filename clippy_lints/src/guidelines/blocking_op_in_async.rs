use clippy_utils::diagnostics::span_lint_and_note;
use clippy_utils::source::snippet_opt;
use clippy_utils::{fn_def_id, is_async_fn};
use rustc_hir::def_id::DefIdSet;
use rustc_hir::intravisit::FnKind;
use rustc_hir::{Block, Body, BodyId, Expr, ExprKind, Guard, Let, Local, Node, StmtKind};
use rustc_lint::LateContext;
use rustc_span::Span;

use super::BLOCKING_OP_IN_ASYNC;

/// Basic list of functions' path to check for
pub(super) static FUNCTIONS_BLACKLIST: &[&[&str]] = &[&["std", "thread", "sleep"]];
/// Functions in these slice will be checked if `allow-io-blocking-ops` option
/// was set to `false` in user configuration.
pub(super) static IO_FUNCTIONS_BLACKLIST: &[&[&str]] = &[
    &["std", "fs", "try_exists"],
    &["std", "fs", "canonicalize"],
    &["std", "fs", "copy"],
    &["std", "fs", "create_dir"],
    &["std", "fs", "create_dir_all"],
    &["std", "fs", "hard_link"],
    &["std", "fs", "metadata"],
    &["std", "fs", "read"],
    &["std", "fs", "read_dir"],
    &["std", "fs", "read_link"],
    &["std", "fs", "read_to_string"],
    &["std", "fs", "remove_dir"],
    &["std", "fs", "remove_dir_all"],
    &["std", "fs", "remove_file"],
    &["std", "fs", "rename"],
    &["std", "fs", "set_permissions"],
    &["std", "fs", "symlink_metadata"],
    &["std", "fs", "write"],
    &["std", "io", "copy"],
    &["std", "io", "empty"],
    &["std", "io", "read_to_string"],
    &["std", "io", "repeat"],
    &["std", "io", "sink"],
    &["std", "io", "stderr"],
    &["std", "io", "stdin"],
    &["std", "io", "stdout"],
];

pub(super) fn check_fn(cx: &LateContext<'_>, kind: FnKind<'_>, body: &Body<'_>, span: Span, blacklist_ids: &DefIdSet) {
    if !is_async_fn(kind) {
        return;
    }

    let decl_span = cx.tcx.sess.source_map().guess_head_span(span);
    lint_fn_calls(cx, body.value, blacklist_ids, decl_span);
}

pub(super) fn check_closure(cx: &LateContext<'_>, expr: &Expr<'_>, blacklist_ids: &DefIdSet) {
    if let ExprKind::Closure(closure) = &expr.kind {
        let Some(async_span) = get_async_span(cx, expr.span) else { return; };

        if let Some(closure_body) = body_expr_from_body_id(cx, closure.body) {
            lint_fn_calls(cx, closure_body, blacklist_ids, async_span);
        }
    }
}

/// Return the `async` keyword span for a closure if it starts with one.
fn get_async_span(cx: &LateContext<'_>, span: Span) -> Option<Span> {
    let start_span = cx.tcx.sess.source_map().span_until_whitespace(span);

    if snippet_opt(cx, start_span)
        .filter(|snippet| snippet == "async")
        .is_some()
    {
        Some(start_span)
    } else {
        None
    }
}

fn body_expr_from_body_id<'tcx>(cx: &'tcx LateContext<'_>, body: BodyId) -> Option<&'tcx Expr<'tcx>> {
    let node = cx.tcx.hir().find(body.hir_id)?;

    if let Node::Expr(expr) = node {
        match &expr.kind {
            ExprKind::Closure(closure) => body_expr_from_body_id(cx, closure.body),
            _ => Some(expr),
        }
    } else {
        None
    }
}

fn lint_fn_calls(cx: &LateContext<'_>, expr: &Expr<'_>, blacklist_ids: &DefIdSet, decl_span: Span) {
    match &expr.kind {
        ExprKind::Call(_, [arg, ..]) => {
            if let ExprKind::MethodCall(_, caller, ..) = &arg.kind {
                lint_fn_calls(cx, caller, blacklist_ids, decl_span);
            } else {
                emit_lint_on_blacklisted_fns(cx, expr, blacklist_ids, decl_span);
            }
        },
        ExprKind::Closure(closure) => {
            // This closure should already in an async context at this point,
            // so no need to check for asyncness again.
            if let Some(closure_body) = body_expr_from_body_id(cx, closure.body) {
                lint_fn_calls(cx, closure_body, blacklist_ids, decl_span);
            }
        },
        // chained methods, e.g. `std::fs::read(..).map(..).map_err(..)`
        ExprKind::MethodCall(_, caller, ..) => lint_fn_calls(cx, caller, blacklist_ids, decl_span),
        // e.g. `{ std::thread::sleep(..) }`
        ExprKind::DropTemps(e) => lint_fn_calls(cx, e, blacklist_ids, decl_span),
        // e.g. `let x = std::fs::read(..)`
        ExprKind::Let(Let { init, .. }) => lint_fn_calls(cx, init, blacklist_ids, decl_span),
        // match clauses, e.g. `match std::fs::read(..) {..}` or `read(..)?`
        ExprKind::Match(mat_expr, arms, ..) => {
            lint_fn_calls(cx, mat_expr, blacklist_ids, decl_span);
            for arm in *arms {
                match arm.guard {
                    Some(Guard::If(e)) => lint_fn_calls(cx, e, blacklist_ids, decl_span),
                    Some(Guard::IfLet(Let { init, .. })) => lint_fn_calls(cx, init, blacklist_ids, decl_span),
                    None => (),
                }
                lint_fn_calls(cx, arm.body, blacklist_ids, decl_span);
            }
        },
        // e.g. `if read(..)` or `if let .. = read(..)`
        ExprKind::If(if_clause, then, maybe_else) => {
            lint_fn_calls(cx, if_clause, blacklist_ids, decl_span);
            lint_fn_calls(cx, then, blacklist_ids, decl_span);
            if let Some(else_clause) = maybe_else {
                lint_fn_calls(cx, else_clause, blacklist_ids, decl_span);
            }
        },
        ExprKind::Block(
            Block {
                stmts, expr: maybe_ret, ..
            },
            None,
        ) => {
            if let Some(ret_expr) = maybe_ret {
                lint_fn_calls(cx, ret_expr, blacklist_ids, decl_span);
            }
            for stmt in *stmts {
                match stmt.kind {
                    StmtKind::Local(Local {
                        init: Some(init_expr), ..
                    }) => lint_fn_calls(cx, init_expr, blacklist_ids, decl_span),
                    StmtKind::Semi(e) | StmtKind::Expr(e) => lint_fn_calls(cx, e, blacklist_ids, decl_span),
                    _ => (),
                }
            }
        },
        _ => (),
    }
}

fn emit_lint_on_blacklisted_fns(cx: &LateContext<'_>, expr: &Expr<'_>, blacklist_ids: &DefIdSet, decl_span: Span) {
    if let Some(did) = fn_def_id(cx, expr) {
        if blacklist_ids.contains(&did) {
            span_lint_and_note(
                cx,
                BLOCKING_OP_IN_ASYNC,
                expr.span,
                "this function call might blocks the thread in async context",
                Some(decl_span),
                "asyncness was determined here",
            );
        }
    }
}
