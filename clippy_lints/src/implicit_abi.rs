use clippy_utils::diagnostics::span_lint_and_sugg;
use rustc_ast::ast::{Item, ItemKind};
use rustc_errors::Applicability;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_lint_pass, declare_tool_lint};

declare_clippy_lint! {
    /// ### What it does
    /// Checks the external block without explicitly lable its ABI.
    ///
    /// ### Why is this bad?
    /// Implicit ABI has negative impact on code readability.
    ///
    /// ### Example
    /// ```rust
    /// extern {
    ///     fn c_function();
    /// }
    /// ```
    /// Use instead:
    /// ```rust
    /// extern "C" {
    ///     fn c_function();
    /// }
    /// ```
    #[clippy::version = "1.70.0"]
    pub IMPLICIT_ABI,
    restriction,
    "external block with implicit ABI"
}

declare_lint_pass!(ImplicitAbi => [IMPLICIT_ABI]);

impl EarlyLintPass for ImplicitAbi {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
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
}
