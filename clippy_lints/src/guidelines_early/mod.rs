mod implicit_abi;
mod infinite_loop;

use rustc_ast::ast;
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_session::{declare_tool_lint, impl_lint_pass};

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

declare_clippy_lint! {
    /// ### What it does
    /// Checks for the existance of infinite loop.
    ///
    /// ### Why is this bad?
    /// This could be an error where the programmer forgets to add an exit mechanism,
    /// thus have the risk of draining resources during runtime.
    ///
    /// ### Known problems
    ///
    /// In some cases, such as during server communication or signal handling, where
    /// using infinite loops could be as intended.
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
    pub INFINITE_LOOP,
    nursery,
    "loop block without `break` or `return` statement"
}

#[derive(Clone, Default)]
pub struct LintGroup;

impl_lint_pass!(LintGroup => [
    IMPLICIT_ABI,
    INFINITE_LOOP,
]);

impl EarlyLintPass for LintGroup {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &ast::Item) {
        implicit_abi::check(cx, item);
    }

    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &ast::Expr) {
        infinite_loop::check(cx, expr);
    }
}
