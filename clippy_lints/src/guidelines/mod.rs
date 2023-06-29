mod blocking_op_in_async;
mod falliable_memory_allocation;
mod mem_unsafe_functions;
mod passing_string_to_c_functions;
mod unsafe_block_in_proc_macro;
mod untrusted_lib_loading;

use clippy_utils::def_path_def_ids;
use rustc_data_structures::fx::FxHashSet;
use rustc_hir as hir;
use rustc_hir::def_id::{DefId, DefIdSet};
use rustc_hir::intravisit;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_tool_lint, impl_lint_pass};
use rustc_span::def_id::LocalDefId;
use rustc_span::Span;

declare_clippy_lint! {
    /// ### What it does
    /// Checks for direct usage of external functions that modify memory
    /// without concerning about memory safety, such as `memcpy`, `strcpy`, `strcat` etc.
    ///
    /// ### Why is this bad?
    /// These function can be dangerous when used incorrectly,
    /// which could potentially introduce vulnerablities such as buffer overflow to the software.
    ///
    /// ### Example
    /// ```rust
    /// extern "C" {
    ///     fn memcpy(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void;
    /// }
    /// let ptr = unsafe { memcpy(dest, src, size); }
    /// // Or use via libc
    /// let ptr = unsafe { libc::memcpy(dest, src, size); }
    #[clippy::version = "1.70.0"]
    pub MEM_UNSAFE_FUNCTIONS,
    nursery,
    "use of potentially dangerous external functions"
}

declare_clippy_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Example
    /// ```rust
    /// // example code where clippy issues a warning
    /// ```
    /// Use instead:
    /// ```rust
    /// // example code which does not raise clippy warning
    /// ```
    #[clippy::version = "1.70.0"]
    pub UNTRUSTED_LIB_LOADING,
    nursery,
    "attempt to load dynamic library from untrusted source"
}

declare_clippy_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Example
    /// ```rust
    /// // example code where clippy issues a warning
    /// ```
    /// Use instead:
    /// ```rust
    /// // example code which does not raise clippy warning
    /// ```
    #[clippy::version = "1.70.0"]
    pub PASSING_STRING_TO_C_FUNCTIONS,
    nursery,
    "passing string or str to extern C function"
}

declare_clippy_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Example
    /// ```rust
    /// // example code where clippy issues a warning
    /// ```
    /// Use instead:
    /// ```rust
    /// // example code which does not raise clippy warning
    /// ```
    #[clippy::version = "1.70.0"]
    pub FALLIABLE_MEMORY_ALLOCATION,
    nursery,
    "memory allocation without checking arguments and result"
}

declare_clippy_lint! {
    /// ### What it does
    /// Checks for calling certain function that could block its thread in an async context.
    ///
    /// ### Why is this bad?
    /// Blocking a thread prevents tasks being swapped, causing other tasks to stop running
    /// until the thread is no longer blocked, which might lead to unexpected behavior.
    ///
    /// ### Example
    /// ```rust
    /// use std::time::Duration;
    /// pub async fn foo() {
    ///     std::thread::sleep(Duration::from_secs(5));
    /// }
    /// ```
    /// Use instead:
    /// ```rust
    /// use std::time::Duration;
    /// pub async fn foo() {
    ///     tokio::time::sleep(Duration::from_secs(5));
    /// }
    /// ```
    #[clippy::version = "1.70.0"]
    pub BLOCKING_OP_IN_ASYNC,
    nursery,
    "calling blocking funtions in an async context"
}

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

/// Helper struct with user configured path-like functions, such as `std::fs::read`,
/// and a set for `def_id`s which should be filled during checks.
///
/// NB: They might not have a one-on-one relation.
#[derive(Clone, Default)]
pub struct FnPathsAndIds {
    pub paths: Vec<String>,
    pub ids: DefIdSet,
}

impl FnPathsAndIds {
    fn with_paths(paths: Vec<String>) -> Self {
        Self {
            paths,
            ids: DefIdSet::new(),
        }
    }
}

#[derive(Clone, Default)]
pub struct GuidelineLints {
    mem_uns_fns: FnPathsAndIds,
    io_fns: FnPathsAndIds,
    blocking_fns: FnPathsAndIds,
    allow_io_blocking_ops: bool,
    macro_call_sites: FxHashSet<Span>,
}

impl GuidelineLints {
    pub fn new(mem_uns_fns: Vec<String>, io_fns: Vec<String>, allow_io_blocking_ops: bool) -> Self {
        Self {
            mem_uns_fns: FnPathsAndIds::with_paths(mem_uns_fns),
            io_fns: FnPathsAndIds::with_paths(io_fns),
            blocking_fns: FnPathsAndIds::default(),
            allow_io_blocking_ops,
            macro_call_sites: FxHashSet::default(),
        }
    }
}

impl_lint_pass!(GuidelineLints => [
    MEM_UNSAFE_FUNCTIONS,
    UNTRUSTED_LIB_LOADING,
    PASSING_STRING_TO_C_FUNCTIONS,
    FALLIABLE_MEMORY_ALLOCATION,
    BLOCKING_OP_IN_ASYNC,
    UNSAFE_BLOCK_IN_PROC_MACRO,
]);

impl<'tcx> LateLintPass<'tcx> for GuidelineLints {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        kind: intravisit::FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'_>,
        body: &'tcx hir::Body<'_>,
        span: Span,
        _def_id: LocalDefId,
    ) {
        if !matches!(kind, intravisit::FnKind::Closure) {
            blocking_op_in_async::check_fn(cx, kind, body, span, &self.blocking_fns.ids);
        }
    }

    fn check_crate(&mut self, cx: &LateContext<'tcx>) {
        // Resolve function names to def_ids from configuration
        for uns_fns in &self.mem_uns_fns.paths {
            // Path like function names such as `libc::foo` or `aa::bb::cc::bar`,
            // this only works with dependencies.
            if uns_fns.contains("::") {
                let path: Vec<&str> = uns_fns.split("::").collect();
                for did in def_path_def_ids(cx, path.as_slice()) {
                    self.mem_uns_fns.ids.insert(did);
                }
            }
            // Plain function names, then we should take its libc variant into account
            else if let Some(did) = libc_fn_def_id(cx, uns_fns) {
                self.mem_uns_fns.ids.insert(did);
            }
        }

        blocking_op_in_async::init_blacklist_ids(cx, self.allow_io_blocking_ops, &mut self.blocking_fns.ids);
    }

    fn check_item(&mut self, _cx: &LateContext<'tcx>, item: &'tcx hir::Item<'_>) {
        mem_unsafe_functions::check_foreign_item(item, &self.mem_uns_fns.paths, &mut self.mem_uns_fns.ids);
    }

    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'_>) {
        untrusted_lib_loading::check(cx, expr, &self.io_fns.paths);
        passing_string_to_c_functions::check_expr(cx, expr);
        falliable_memory_allocation::check_expr(cx, expr);
        mem_unsafe_functions::check(cx, expr, &self.mem_uns_fns.ids);
        blocking_op_in_async::check_closure(cx, expr, &self.blocking_fns.ids);
        unsafe_block_in_proc_macro::check(cx, expr, &mut self.macro_call_sites);
    }
}

fn libc_fn_def_id(cx: &LateContext<'_>, fn_name: &str) -> Option<DefId> {
    let path = &["libc", fn_name];
    def_path_def_ids(cx, path).next()
}
