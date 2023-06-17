use clippy_utils::diagnostics::span_lint;
use rustc_ast::ast::{Expr, ExprKind, Path};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_session::{declare_lint_pass, declare_tool_lint};

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
declare_lint_pass!(NonReentrantFunctions => [NON_REENTRANT_FUNCTIONS]);

impl EarlyLintPass for NonReentrantFunctions {
    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        if expr.span.from_expansion() {
            return;
        }

        let msg: &str = "consider using the reentrant version of the function";

        match expr.kind {
            ExprKind::Call(ref func, _) => {
                if is_reentrant_fn(func) {
                    span_lint(cx, NON_REENTRANT_FUNCTIONS, expr.span, msg);
                }
            },
            _ => {},
        }
    }
}

fn is_reentrant_fn(func: &Expr) -> bool {
    match &func.kind {
        ExprKind::Path(_, Path { segments, .. }) => {
            let seg = segments.iter().last().unwrap();
            let ident = format!("{:?}", seg.ident);
            check_reentrant_by_fn_name(&ident)
        },
        _ => false,
    }
}

fn check_reentrant_by_fn_name(func: &str) -> bool {
    let name = func.split("#").next().unwrap();
    name == "strtok" || name == "localtime"
}
