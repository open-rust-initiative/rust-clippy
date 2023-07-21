use super::LOOP_WITHOUT_BREAK_OR_RETURN;
use clippy_utils::diagnostics::span_lint_and_help;
use rustc_ast::ast::{Block, Expr, ExprKind, Label, StmtKind};
use rustc_lint::EarlyContext;

pub(super) fn check(cx: &EarlyContext<'_>, expr: &Expr) {
    if let ExprKind::Loop(block, label, _) = &expr.kind {
        if !check_block(block, label, true) {
            span_lint_and_help(
                cx,
                LOOP_WITHOUT_BREAK_OR_RETURN,
                expr.span,
                "loop without break condition",
                None,
                "consider adding `break` or `return` statement in the loop block",
            );
        }
    }
}

fn check_block(block: &Block, label: &Option<Label>, outest: bool) -> bool {
    block.stmts.iter().any(|stmt| match &stmt.kind {
        StmtKind::Semi(expr) | StmtKind::Expr(expr) => check_expr(expr, label, outest),
        _ => false,
    })
}

fn check_expr(expr: &Expr, label: &Option<Label>, outest: bool) -> bool {
    match &expr.kind {
        ExprKind::Ret(..) => true,
        ExprKind::Break(lbl, _) => {
            if outest {
                true
            } else {
                label.is_some() && label == lbl
            }
        },
        ExprKind::If(_, blk, else_expr) => {
            let mut do_exit = check_block(blk, label, outest);
            if let Some(expr) = else_expr {
                do_exit = do_exit || check_expr(expr, label, outest);
            }
            do_exit
        },
        ExprKind::Loop(blk, ..) | ExprKind::ForLoop(_, _, blk, _) | ExprKind::While(_, blk, _) => {
            check_block(blk, label, false)
        },
        ExprKind::Block(blk, _) | ExprKind::Async(_, blk) => check_block(blk, label, outest),
        ExprKind::Match(_, arms) => arms.iter().any(|arm| check_expr(&arm.body, label, outest)),
        _ => false,
    }
}
