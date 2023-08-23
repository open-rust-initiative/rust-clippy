use super::INFINITE_LOOP;
use clippy_utils::diagnostics::span_lint_and_help;
use rustc_ast::ast::{Expr, ExprKind, Label};
use rustc_ast::visit::{walk_expr, Visitor};
use rustc_lint::EarlyContext;

pub(super) fn check(cx: &EarlyContext<'_>, expr: &Expr) {
    if let ExprKind::Loop(block, label, _) = &expr.kind {
        // First, find any `break` or `return` without entering any inner loop,
        // then, find `return` or labeled `break` which breaks this loop with entering inner loop,
        // otherwise this loop is a infinite loop.
        let mut direct_br_or_ret_finder = DirectBreakOrRetFinder::default();
        direct_br_or_ret_finder.visit_block(block);

        let is_finite_loop = if direct_br_or_ret_finder.found {
            true
        } else if let Some(lbl) = label {
            let mut inner_br_or_ret_finder = InnerBreakOrRetFinder::with_label(*lbl);
            inner_br_or_ret_finder.visit_block(block);
            inner_br_or_ret_finder.found
        } else {
            false
        };

        if !is_finite_loop {
            span_lint_and_help(
                cx,
                INFINITE_LOOP,
                expr.span,
                "loop without break condition",
                None,
                "consider adding `break` or `return` statement in the loop block",
            );
        }
    }
}

/// Find direct `break` or `return` without entering sub loop.
#[derive(Default)]
struct DirectBreakOrRetFinder {
    found: bool,
}

impl<'ast> Visitor<'ast> for DirectBreakOrRetFinder {
    fn visit_expr(&mut self, ex: &'ast Expr) {
        match &ex.kind {
            ExprKind::Break(..) | ExprKind::Ret(..) => self.found = true,
            ExprKind::Loop(..) | ExprKind::While(..) | ExprKind::ForLoop(..) => (),
            _ => walk_expr(self, ex),
        }
    }
}

/// Find `break` or `return` with entering inner loops, and find a break with corresponding label
struct InnerBreakOrRetFinder {
    label: Label,
    found: bool,
}

impl InnerBreakOrRetFinder {
    fn with_label(label: Label) -> Self {
        Self { label, found: false }
    }
}

impl<'ast> Visitor<'ast> for InnerBreakOrRetFinder {
    fn visit_expr(&mut self, ex: &'ast Expr) {
        match &ex.kind {
            ExprKind::Break(maybe_label, ..) => {
                if let Some(break_label) = maybe_label {
                    if self.label == *break_label {
                        self.found = true;
                    }
                }
            },
            ExprKind::Ret(..) => self.found = true,
            _ => walk_expr(self, ex),
        }
    }
}
