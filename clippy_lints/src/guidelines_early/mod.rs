mod implicit_abi;
mod loop_without_break_or_return;
mod non_reentrant_functions;

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

declare_clippy_lint! {
    /// ### What it does
    /// Checks for non-reentrant functions.
    ///
    /// ### Why is this bad?
    /// This makes code safer, especially in the context of concurrency.
    ///
    /// ### Example
    /// ```rust
    /// let _tm = libc::localtime(&0i64 as *const libc::time_t);
    /// ```
    /// Use instead:
    /// ```rust
    /// let res = libc::malloc(std::mem::size_of::<libc::tm>());
    ///
    /// libc::locatime_r(&0i64 as *const libc::time_t, res);
    /// ```
    #[clippy::version = "1.70.0"]
    pub NON_REENTRANT_FUNCTIONS,
    nursery,
    "this function is a non-reentrant-function"
}

#[derive(Clone, Default)]
pub struct LintGroup;

impl_lint_pass!(LintGroup => [
    IMPLICIT_ABI,
    LOOP_WITHOUT_BREAK_OR_RETURN,
    NON_REENTRANT_FUNCTIONS,
]);

impl EarlyLintPass for LintGroup {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &ast::Item) {
        implicit_abi::check(cx, item);
    }

    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &ast::Expr) {
        if !expr.span.from_expansion() {
            loop_without_break_or_return::check(cx, expr);
            non_reentrant_functions::check(cx, expr);
        }
    }
}
