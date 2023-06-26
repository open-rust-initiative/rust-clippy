use clippy_utils::diagnostics::span_lint_and_help;
use clippy_utils::source::snippet_opt;
use if_chain::if_chain;
use rustc_data_structures::fx::FxHashSet;
use rustc_hir::Expr;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_tool_lint, impl_lint_pass};
use rustc_span::hygiene::{ExpnKind, MacroKind};
use rustc_span::{sym, Span};

declare_clippy_lint! {
    /// ### What it does
    /// Checks for unsafe block written in procedural macro
    ///
    /// ### Why is this bad?
    /// It hides the unsafe code, making the safety of expended code unsound.
    ///
    /// ### Known problems
    /// Possible FP when the user uses proc-macro to generate a function with unsafe block in it.
    ///
    /// ### Example
    /// ```rust
    /// #[proc_macro]
    /// pub fn rprintf(input: TokenStream) -> TokenStream {
    ///     let expr = parse_macro_input!(input as syn::Expr);
    ///     quote!({
    ///         unsafe {
    ///             // unsafe operation
    ///         }
    ///     })
    /// }
    ///
    /// // This allows users to use this macro without `unsafe` block
    /// rprintf!();
    /// ```
    /// Use instead:
    /// ```rust
    /// #[proc_macro]
    /// pub fn rprintf(input: TokenStream) -> TokenStream {
    ///     let expr = parse_macro_input!(input as syn::Expr);
    ///     quote!({
    ///         // unsafe operation
    ///     })
    /// }
    ///
    /// // When using this macro, an outer `unsafe` block is needed,
    /// // making the safety of this macro much clearer.
    /// unsafe { rprintf!(); }
    /// ```
    #[clippy::version = "1.70.0"]
    pub UNSAFE_BLOCK_IN_PROC_MACRO,
    nursery,
    "using unsafe block in procedural macro's definition"
}

#[derive(Clone)]
pub struct UnsafeBlockInProcMacro {
    call_sites: FxHashSet<Span>,
}

impl UnsafeBlockInProcMacro {
    pub fn new() -> Self {
        Self {
            call_sites: FxHashSet::default(),
        }
    }
}

impl_lint_pass!(UnsafeBlockInProcMacro => [UNSAFE_BLOCK_IN_PROC_MACRO]);

impl LateLintPass<'_> for UnsafeBlockInProcMacro {
    fn check_expr(&mut self, cx: &LateContext<'_>, expr: &Expr<'_>) {
        let expn_data = expr.span.ctxt().outer_expn_data();
        let call_site = expn_data.call_site;

        if_chain! {
            if !call_site.from_expansion();
            if self.call_sites.insert(call_site);
            if let ExpnKind::Macro(MacroKind::Bang, symbol) = expn_data.kind;
            if symbol == sym::quote;
            if let Some(code_snip) = quote_inner(cx, call_site);
            let could_be_fn = code_snip.contains("fn ");
            if contains_unsafe_block(&code_snip);
            then {
                emit_lint_message(cx, expr, call_site, could_be_fn);
            }
        }
    }
}

/// Get the code snippet inside of `quote!()`
fn quote_inner(cx: &LateContext<'_>, call_site: Span) -> Option<String> {
    let quote_snip = snippet_opt(cx, call_site)?;
    let (_, snip) = quote_snip.split_once('!')?;
    let mut chars = snip.trim().chars();
    chars.next();
    chars.next_back();
    Some(chars.collect())
}

fn contains_unsafe_block(code: &str) -> bool {
    // unify input by eliminating all whitespaces
    let without_ws: String = code.split_whitespace().collect();
    without_ws.contains("unsafe{")
}

fn emit_lint_message(cx: &LateContext<'_>, expr: &Expr<'_>, call_site: Span, could_be_fn: bool) {
    let parent_hid = cx.tcx.hir().parent_id(expr.hir_id);
    let name = cx.tcx.item_name(parent_hid.owner.to_def_id());
    let msg = &format!("procedural macro `{name}` may hiding unsafe operations");
    let extra_sugg = if could_be_fn {
        "if the block was in a function, \
        put an `unsafe` keyword on its definition: `unsafe fn ...`, \n\
        otherwise"
    } else {
        "and"
    };
    let help = &format!(
        "consider removing unsafe block from the above `quote!` call,\n\
        {extra_sugg} add unsafe block when calling the macro instead: `unsafe {{ {name}!{{...}} }}`"
    );

    span_lint_and_help(cx, UNSAFE_BLOCK_IN_PROC_MACRO, call_site, msg, None, help);
}
