use std::ops::ControlFlow;

use clippy_utils::diagnostics::span_lint_and_note;
use clippy_utils::source::snippet_opt;
use clippy_utils::visitors::{for_each_expr_with_closures, Visitable};
use clippy_utils::{def_path_def_ids, fn_def_id, is_async_fn};
use rustc_hir::def_id::DefIdSet;
use rustc_hir::intravisit::FnKind;
use rustc_hir::{Body, Closure, Expr, ExprKind, Node};
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

pub(super) fn check_fn<'tcx>(
    cx: &LateContext<'tcx>,
    kind: FnKind<'_>,
    body: &'tcx Body<'_>,
    span: Span,
    blacklist_ids: &DefIdSet,
) {
    if !is_async_fn(kind) {
        return;
    }
    let decl_span = cx.tcx.sess.source_map().guess_head_span(span);
    lint_blacklisted_call(cx, body, blacklist_ids, decl_span);
}

pub(super) fn check_expr<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>, blacklist_ids: &DefIdSet) {
    if let ExprKind::Closure(Closure { body, .. }) = expr.kind &&
        let Some(body_node) = cx.tcx.hir().find(body.hir_id) &&
        let Node::Expr(body_expr) = body_node &&
        let Some(async_span) = get_async_span(cx, expr.span)
    {
        lint_blacklisted_call(cx, body_expr, blacklist_ids, async_span);
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

fn lint_blacklisted_call<'tcx>(
    cx: &LateContext<'tcx>,
    node: impl Visitable<'tcx>,
    blacklist_ids: &DefIdSet,
    note_span: Span,
) {
    let mut blocking_call_spans = vec![];
    for_each_expr_with_closures(cx, node, |e| {
        if let Some(did) = fn_def_id(cx, e) && blacklist_ids.contains(&did) {
            blocking_call_spans.push(e.span);
        }
        ControlFlow::<()>::Continue(())
    });
    for call_span in blocking_call_spans {
        span_lint_and_note(
            cx,
            BLOCKING_OP_IN_ASYNC,
            call_span,
            "this call might blocks the thread in async context",
            Some(note_span),
            "asyncness was determined here",
        );
    }
}
