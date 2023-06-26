use clippy_utils::diagnostics::span_lint;
use rustc_ast::ast::{Block, Expr, ExprKind, Label, StmtKind};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_session::{declare_lint_pass, declare_tool_lint};

declare_clippy_lint! {
    /// ### What it does
    /// Checks for loop-without-exit-mechanism.
    ///
    /// ### Why is this bad?
    /// This makes code bug-prone.
    ///
    /// ### Example
    /// ```rust
    /// loop {
    ///     println!("so something");
    /// }
    /// ```
    /// Use instead:
    /// ```rust
    /// loop {
    ///     println!("do something");
    ///     if flag {
    ///         break;
    ///     }
    /// }
    /// ```
    #[clippy::version = "1.70.0"]
    pub LOOP_WITHOUT_BREAK_OR_RETURN,
    nursery,
    "loop block without `break` or `return` statement"
}
declare_lint_pass!(LoopWithoutBreakOrReturn => [LOOP_WITHOUT_BREAK_OR_RETURN]);

impl EarlyLintPass for LoopWithoutBreakOrReturn {
    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        if expr.span.from_expansion() {
            return;
        }

        let msg: &str = "consider adding `break` or `return` statement in the loop block";

        if let ExprKind::Loop(block, label, _) = &expr.kind {
            if !check_block(block, label, true) {
                span_lint(cx, LOOP_WITHOUT_BREAK_OR_RETURN, expr.span, msg);
            }
        }
    }
}

fn check_block(block: &Block, label: &Option<Label>, outest: bool) -> bool {
    block.stmts.iter().any(|stmt| match &stmt.kind {
        StmtKind::Semi(expr) | StmtKind::Expr(expr) => !expr.span.from_expansion() && check_expr(expr, label, outest),
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
