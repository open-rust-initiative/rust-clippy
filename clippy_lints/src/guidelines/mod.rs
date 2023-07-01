mod blocking_op_in_async;
mod fallible_memory_allocation;
mod mem_unsafe_functions;
mod passing_string_to_c_functions;
mod unsafe_block_in_proc_macro;
mod untrusted_lib_loading;

use clippy_utils::def_path_def_ids;
use rustc_data_structures::fx::FxHashSet;
use rustc_hir as hir;
use rustc_hir::def_id::DefIdSet;
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
    pub FALLIBLE_MEMORY_ALLOCATION,
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
    /// additional external memory allocation function names
    /// other than [`fallible_memory_allocation::DEFAULT_MEM_ALLOC_FNS`]
    mem_alloc_fns: FnPathsAndIds,
    io_fns: FnPathsAndIds,
    blocking_fns: FnPathsAndIds,
    allow_io_blocking_ops: bool,
    macro_call_sites: FxHashSet<Span>,
    /// additional checker function names
    /// other than [`fallible_memory_allocation::DEFAULT_ALLOC_SIZE_CHECK_FNS`]
    alloc_size_check_fns: Vec<String>,
}

impl GuidelineLints {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(
        mem_uns_fns: Vec<String>,
        io_fns: Vec<String>,
        allow_io_blocking_ops: bool,
        alloc_size_check_fns: Vec<String>,
        mem_alloc_fns: Vec<String>,
    ) -> Self {
        let mut all_checker_fns = str_slice_owned(fallible_memory_allocation::DEFAULT_ALLOC_SIZE_CHECK_FNS);
        all_checker_fns.extend_from_slice(&alloc_size_check_fns);
        let mut all_mem_alloc_fns = str_slice_owned(fallible_memory_allocation::DEFAULT_MEM_ALLOC_FNS);
        all_mem_alloc_fns.extend_from_slice(&mem_alloc_fns);

        Self {
            mem_uns_fns: FnPathsAndIds::with_paths(mem_uns_fns),
            mem_alloc_fns: FnPathsAndIds::with_paths(all_mem_alloc_fns),
            io_fns: FnPathsAndIds::with_paths(io_fns),
            blocking_fns: FnPathsAndIds::default(),
            allow_io_blocking_ops,
            macro_call_sites: FxHashSet::default(),
            alloc_size_check_fns: all_checker_fns,
        }
    }
}

impl_lint_pass!(GuidelineLints => [
    MEM_UNSAFE_FUNCTIONS,
    UNTRUSTED_LIB_LOADING,
    PASSING_STRING_TO_C_FUNCTIONS,
    FALLIBLE_MEMORY_ALLOCATION,
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
        add_configured_c_fn_ids(cx, &self.mem_uns_fns.paths, &mut self.mem_uns_fns.ids);
        add_configured_c_fn_ids(cx, &self.mem_alloc_fns.paths, &mut self.mem_alloc_fns.ids);
        blocking_op_in_async::init_blacklist_ids(cx, self.allow_io_blocking_ops, &mut self.blocking_fns.ids);
    }

    fn check_item(&mut self, _cx: &LateContext<'tcx>, item: &'tcx hir::Item<'_>) {
        add_extern_fn_ids(item, &self.mem_uns_fns.paths, &mut self.mem_uns_fns.ids);
        add_extern_fn_ids(item, &self.mem_alloc_fns.paths, &mut self.mem_alloc_fns.ids);
    }

    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'_>) {
        untrusted_lib_loading::check(cx, expr, &self.io_fns.paths);
        passing_string_to_c_functions::check_expr(cx, expr);
        fallible_memory_allocation::check_expr(cx, expr, &self.alloc_size_check_fns, &self.mem_alloc_fns.ids);
        mem_unsafe_functions::check(cx, expr, &self.mem_uns_fns.ids);
        blocking_op_in_async::check_closure(cx, expr, &self.blocking_fns.ids);
        unsafe_block_in_proc_macro::check(cx, expr, &mut self.macro_call_sites);
    }
}

/// Convert `&[&str]` to `Vec<String>`
fn str_slice_owned(seq: &[&str]) -> Vec<String> {
    seq.iter().map(ToString::to_string).collect()
}

/// Resolve and insert the `def_id` of user configure extern C functions into `ids`.
fn add_configured_c_fn_ids(cx: &LateContext<'_>, fns: &[String], ids: &mut DefIdSet) {
    for fn_name in fns {
        // Path like function names such as `libc::foo` or `aa::bb::cc::bar`,
        // this only works with dependencies.
        if fn_name.contains("::") {
            let path: Vec<&str> = fn_name.split("::").collect();
            for did in def_path_def_ids(cx, path.as_slice()) {
                ids.insert(did);
            }
        }
        // Plain function names, then we should take its libc variant into account
        else if let Some(did) = def_path_def_ids(cx, &["libc", fn_name]).next() {
            ids.insert(did);
        }
    }
}

/// Insert the `def_id` of external functions into `ids` if those functions were stated in the `fns`
/// slice.
fn add_extern_fn_ids(item: &hir::Item<'_>, fns: &[String], ids: &mut DefIdSet) {
    if let hir::ItemKind::ForeignMod { items, .. } = item.kind {
        for f_item in items {
            if fns.contains(&f_item.ident.as_str().to_string()) {
                let f_did = f_item.id.hir_id().owner.def_id.to_def_id();
                ids.insert(f_did);
            }
        }
    }
}
