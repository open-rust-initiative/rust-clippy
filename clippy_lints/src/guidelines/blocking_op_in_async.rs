use clippy_utils::diagnostics::span_lint_and_note;
use clippy_utils::source::snippet_opt;
use clippy_utils::{def_path_def_ids, fn_def_id, is_async_fn};
use rustc_hir::def_id::DefIdSet;
use rustc_hir::intravisit::FnKind;
use rustc_hir::{Block, Body, Closure, Expr, ExprKind, Guard, Let, Local, Node, StmtKind};
use rustc_lint::LateContext;
use rustc_span::Span;

use super::BLOCKING_OP_IN_ASYNC;

/// Basic list of functions' path to check for
static FUNCTIONS_BLACKLIST: &[&[&str]] = &[&["std", "thread", "sleep"]];
/// Functions in these slice will be checked if `allow-io-blocking-ops` option
/// was set to `false` in user configuration.
static IO_FUNCTIONS_BLACKLIST: &[&[&str]] = &[
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

pub(super) fn init_blacklist_ids(cx: &LateContext<'_>, allow_io_blocking_ops: bool, blacklist_ids: &mut DefIdSet) {
    let mut insert_did = |list: &[&[&str]]| {
        for fn_path in list {
            for did in def_path_def_ids(cx, fn_path) {
                blacklist_ids.insert(did);
            }
        }
    };

    insert_did(FUNCTIONS_BLACKLIST);
    if !allow_io_blocking_ops {
        insert_did(IO_FUNCTIONS_BLACKLIST);
    }
}

pub(super) fn check_fn(cx: &LateContext<'_>, kind: FnKind<'_>, body: &Body<'_>, span: Span, blacklist_ids: &DefIdSet) {
    if !is_async_fn(kind) {
        return;
    }
    let decl_span = cx.tcx.sess.source_map().guess_head_span(span);
    look_for_call(cx, body.value, blacklist_ids, decl_span);
}

pub(super) fn check_closure(cx: &LateContext<'_>, expr: &Expr<'_>, blacklist_ids: &DefIdSet) {
    if let ExprKind::Closure(closure) = &expr.kind {
        let Some(async_span) = get_async_span(cx, expr.span) else { return; };
        look_for_call_in_closure(cx, closure, blacklist_ids, async_span);
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

fn look_for_call_in_closure(
    cx: &LateContext<'_>,
    closure: &Closure<'_>,
    blacklist_ids: &DefIdSet,
    decl_span: Span,
) {
    let body_expr = cx.tcx.hir().find(closure.body.hir_id);

    if let Some(Node::Expr(expr)) = body_expr {
        match &expr.kind {
            ExprKind::Closure(inner) => look_for_call_in_closure(cx, inner, blacklist_ids, decl_span),
            _ => look_for_call(cx, expr, blacklist_ids, decl_span),
        }
    }
}

/// Recursively look for [`ExprKind::Call`] to see if it matches any blacklisted functions.
fn look_for_call(cx: &LateContext<'_>, expr: &Expr<'_>, blacklist_ids: &DefIdSet, decl_span: Span) {
    match &expr.kind {
        ExprKind::Call(_, args) => {
            emit_lint_on_blacklisted_fns(cx, expr, blacklist_ids, decl_span);
            if let Some(arg) = args.first() {
                look_for_call(cx, arg, blacklist_ids, decl_span);
            }
        },
        // closure block
        ExprKind::Closure(closure) => look_for_call_in_closure(cx, closure, blacklist_ids, decl_span),
        // chained methods, e.g. `std::fs::read(..).map(..).map_err(..)`
        ExprKind::MethodCall(_, caller, ..) => look_for_call(cx, caller, blacklist_ids, decl_span),
        // e.g. `{ std::thread::sleep(..) }`
        ExprKind::DropTemps(e) => look_for_call(cx, e, blacklist_ids, decl_span),
        // e.g. `let x = std::fs::read(..)`
        ExprKind::Let(Let { init, .. }) => look_for_call(cx, init, blacklist_ids, decl_span),
        // e.g. `if read(..)` or `if let .. = read(..)`
        ExprKind::If(cond, then, maybe_else) => {
            look_for_call(cx, cond, blacklist_ids, decl_span);
            look_for_call(cx, then, blacklist_ids, decl_span);
            if let Some(else_clause) = maybe_else {
                look_for_call(cx, else_clause, blacklist_ids, decl_span);
            }
        },
        // match clauses, e.g. `match std::fs::read(..) {..}` or `read(..)?`
        ExprKind::Match(mat_expr, arms, ..) => {
            look_for_call(cx, mat_expr, blacklist_ids, decl_span);
            for arm in *arms {
                match arm.guard {
                    Some(Guard::If(e)) => look_for_call(cx, e, blacklist_ids, decl_span),
                    Some(Guard::IfLet(Let { init, .. })) => look_for_call(cx, init, blacklist_ids, decl_span),
                    None => (),
                }
                look_for_call(cx, arm.body, blacklist_ids, decl_span);
            }
        },
        ExprKind::Block(
            Block {
                stmts, expr: maybe_ret, ..
            },
            None,
        ) => {
            if let Some(ret_expr) = maybe_ret {
                look_for_call(cx, ret_expr, blacklist_ids, decl_span);
            }
            for stmt in *stmts {
                match stmt.kind {
                    StmtKind::Local(Local {
                        init: Some(init_expr), ..
                    }) => look_for_call(cx, init_expr, blacklist_ids, decl_span),
                    StmtKind::Semi(e) | StmtKind::Expr(e) => look_for_call(cx, e, blacklist_ids, decl_span),
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
