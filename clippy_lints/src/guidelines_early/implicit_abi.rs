use super::IMPLICIT_ABI;
use clippy_utils::diagnostics::span_lint_and_sugg;
use rustc_ast::ast::{Item, ItemKind};
use rustc_errors::Applicability;
use rustc_lint::{EarlyContext, LintContext};

pub(super) fn check(cx: &EarlyContext<'_>, item: &Item) {
    if let ItemKind::ForeignMod(fm) = &item.kind {
        if fm.abi.is_none() {
            let extern_span = cx.sess().source_map().span_until_whitespace(item.span);
            span_lint_and_sugg(
                cx,
                IMPLICIT_ABI,
                extern_span,
                "missing ABI label on extern block",
                "explicitly states ABI instead",
                "extern \"C\"".to_string(),
                Applicability::MachineApplicable,
            );
        }
    }
}
