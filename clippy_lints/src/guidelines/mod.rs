mod blocking_op_in_async;
mod extern_without_repr;
mod fallible_memory_allocation;
mod null_ptr_dereference;
mod passing_string_to_c_functions;
mod unsafe_block_in_proc_macro;
mod untrusted_lib_loading;

use clippy_utils::diagnostics::span_lint_and_help;
use clippy_utils::{def_path_def_ids, fn_def_id};
use rustc_data_structures::fx::FxHashSet;
use rustc_hir as hir;
use rustc_hir::def_id::DefIdSet;
use rustc_hir::hir_id::HirId;
use rustc_hir::intravisit;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_tool_lint, impl_lint_pass};
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

declare_clippy_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Example
    /// ```rust
    /// struct Foo3 {
    ///     a: libc::c_char,
    ///     b: libc::c_int,
    ///     c: libc::c_longlong,
    /// }
    /// extern "C" fn c_abi_fn4(arg_one: u32, arg_two: *const Foo3) {}
    /// ```
    /// Use instead:
    /// ```rust
    /// #[repr(C)]
    /// struct Foo3 {
    ///     a: libc::c_char,
    ///     b: libc::c_int,
    ///     c: libc::c_longlong,
    /// }
    /// extern "C" fn c_abi_fn4(arg_one: u32, arg_two: *const Foo3) {}
    /// ```
    #[clippy::version = "1.72.0"]
    pub EXTERN_WITHOUT_REPR,
    pedantic,
    "Should use repr to specifing data layout when struct is used in FFI"
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

declare_clippy_lint! {
    /// ### What it does
    /// Checks for raw pointers that are initialized or assigned as null pointers,
    /// but immediately dereferenced without any pre-caution.
    ///
    /// ### Why is this bad?
    /// Dereferencing null pointer is an undefined behavior.
    ///
    /// ### Known problems
    /// This lint only checks direct reference of null pointer, which means if the null pointer
    /// was referenced somewhere before de dereference, this lint would skip it entirely.
    /// For example, if a null pointer was passed to
    /// a function, but that function still does not assign value to its address, then it
    /// would be assumed non-null even though it wasn't.
    ///
    /// ### Example
    /// ```rust
    /// let a: *const i8 = std::ptr::null();
    /// let _ = unsafe { *a };
    /// ```
    ///
    /// Use instead:
    /// ```rust
    /// let a: *const i8 = std::ptr::null();
    /// *a = &10_i8;
    /// let _ = unsafe { *a };
    /// ```
    #[clippy::version = "1.68.0"]
    pub NULL_PTR_DEREFERENCE,
    nursery,
    "Dereferencing null pointers"
}

/// Helper struct with user configured path-like functions, such as `std::fs::read`,
/// and a set for `def_id`s which should be filled during checks.
///
/// NB: They might not have a one-on-one relation.
#[derive(Clone, Default, Debug)]
pub struct FnPathsAndIds {
    pub paths: Vec<String>,
    pub ids: DefIdSet,
}

impl FnPathsAndIds {
    fn with_paths(paths: Vec<String>) -> Self {
        Self {
            paths,
            ..Default::default()
        }
    }
}

#[derive(Clone, Default)]
pub struct LintGroup {
    mem_uns_fns: FnPathsAndIds,
    mem_alloc_fns: FnPathsAndIds,
    io_fns: FnPathsAndIds,
    lib_loading_fns: FnPathsAndIds,
    blocking_fns: FnPathsAndIds,
    non_reentrant_fns: FnPathsAndIds,
    allow_io_blocking_ops: bool,
    macro_call_sites: FxHashSet<Span>,
    alloc_size_check_fns: Vec<String>,
}

impl LintGroup {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(
        mem_uns_fns: Vec<String>,
        io_fns: Vec<String>,
        lib_loading_fns: Vec<String>,
        allow_io_blocking_ops: bool,
        alloc_size_check_fns: Vec<String>,
        mem_alloc_fns: Vec<String>,
        non_reentrant_fns: Vec<String>,
    ) -> Self {
        Self {
            mem_uns_fns: FnPathsAndIds::with_paths(mem_uns_fns),
            mem_alloc_fns: FnPathsAndIds::with_paths(mem_alloc_fns),
            io_fns: FnPathsAndIds::with_paths(io_fns),
            lib_loading_fns: FnPathsAndIds::with_paths(lib_loading_fns),
            non_reentrant_fns: FnPathsAndIds::with_paths(non_reentrant_fns),
            allow_io_blocking_ops,
            alloc_size_check_fns,
            ..Default::default()
        }
    }
}

impl_lint_pass!(LintGroup => [
    MEM_UNSAFE_FUNCTIONS,
    UNTRUSTED_LIB_LOADING,
    PASSING_STRING_TO_C_FUNCTIONS,
    FALLIBLE_MEMORY_ALLOCATION,
    BLOCKING_OP_IN_ASYNC,
    UNSAFE_BLOCK_IN_PROC_MACRO,
    EXTERN_WITHOUT_REPR,
    NON_REENTRANT_FUNCTIONS,
    NULL_PTR_DEREFERENCE,
]);

impl<'tcx> LateLintPass<'tcx> for LintGroup {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        kind: intravisit::FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'_>,
        body: &'tcx hir::Body<'_>,
        span: Span,
        _def_id: HirId,
    ) {
        if !matches!(kind, intravisit::FnKind::Closure) {
            blocking_op_in_async::check_fn(cx, kind, body, span, &self.blocking_fns.ids);
        }
    }

    fn check_crate(&mut self, cx: &LateContext<'tcx>) {
        add_configured_fn_ids(cx, &mut self.mem_uns_fns);
        add_configured_fn_ids(cx, &mut self.mem_alloc_fns);
        add_configured_fn_ids(cx, &mut self.io_fns);
        add_configured_fn_ids(cx, &mut self.lib_loading_fns);
        add_configured_fn_ids(cx, &mut self.non_reentrant_fns);

        blocking_op_in_async::init_blacklist_ids(cx, self.allow_io_blocking_ops, &mut self.blocking_fns.ids);
    }

    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'_>) {
        if let hir::ItemKind::ForeignMod { items, .. } = item.kind {
            add_extern_fn_ids(items, &mut self.mem_uns_fns);
            add_extern_fn_ids(items, &mut self.mem_alloc_fns);
            add_extern_fn_ids(items, &mut self.io_fns);
            add_extern_fn_ids(items, &mut self.lib_loading_fns);
            add_extern_fn_ids(items, &mut self.non_reentrant_fns);
        }
        extern_without_repr::check_item(cx, item);
    }

    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'_>) {
        if let hir::ExprKind::Call(_func, params) = &expr.kind {
            if let Some(fn_did) = fn_def_id(cx, expr) {
                if self.non_reentrant_fns.ids.contains(&fn_did) {
                    span_lint_and_help(
                        cx,
                        NON_REENTRANT_FUNCTIONS,
                        expr.span,
                        "use of non-reentrant function",
                        None,
                        "consider using its reentrant counterpart",
                    );
                } else if self.mem_uns_fns.ids.contains(&fn_did) {
                    span_lint_and_help(
                        cx,
                        MEM_UNSAFE_FUNCTIONS,
                        expr.span,
                        "use of potentially dangerous memory manipulation function",
                        None,
                        "consider using its safe version",
                    );
                } else if self.lib_loading_fns.ids.contains(&fn_did) {
                    untrusted_lib_loading::check_expr(cx, expr, params, &self.io_fns.ids);
                } else if self.mem_alloc_fns.ids.contains(&fn_did) {
                    fallible_memory_allocation::check_expr(cx, expr, params, fn_did, &self.alloc_size_check_fns);
                }
                passing_string_to_c_functions::check_expr(cx, expr, fn_did, params);
            }
        } else {
            blocking_op_in_async::check_expr(cx, expr, &self.blocking_fns.ids);
            unsafe_block_in_proc_macro::check(cx, expr, &mut self.macro_call_sites);
            null_ptr_dereference::check_assign(cx, expr);
        }
    }

    fn check_local(&mut self, cx: &LateContext<'tcx>, local: &'tcx hir::Local<'tcx>) {
        null_ptr_dereference::check_local(cx, local);
    }
}

/// Resolve and insert the `def_id` of user configure functions if:
///
/// 1. They are the full path like string, such as: `krate::module::func`.
/// 2. They are function names in libc crate.
fn add_configured_fn_ids(cx: &LateContext<'_>, fns: &mut FnPathsAndIds) {
    for fn_name in &fns.paths {
        // Path like function names such as `libc::foo` or `aa::bb::cc::bar`,
        // this only works with dependencies.
        if fn_name.contains("::") {
            let path: Vec<&str> = fn_name.split("::").collect();
            for did in def_path_def_ids(cx, path.as_slice()) {
                fns.ids.insert(did);
            }
        }
        // Plain function names, then we should take its libc variant into account
        else {
            for did in def_path_def_ids(cx, &["libc", fn_name]) {
                fns.ids.insert(did);
            }
        }
    }
}

/// Resolve and insert the `def_id` of functions declared in an `extern` block
fn add_extern_fn_ids(items: &[hir::ForeignItemRef], fns: &mut FnPathsAndIds) {
    for f_item in items {
        if fns.paths.contains(&f_item.ident.as_str().to_string()) {
            let f_did = f_item.id.hir_id().owner.def_id.to_def_id();
            fns.ids.insert(f_did);
        }
    }
}
