mod falliable_memory_allocation;
mod mem_unsafe_functions;
mod passing_string_to_c_functions;
mod untrusted_lib_loading;

use rustc_hir as hir;
use rustc_hir::intravisit;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint_pass, declare_tool_lint};
use rustc_span::def_id::LocalDefId;
use rustc_span::Span;

declare_clippy_lint! {
    /// ### What it does
    /// Check for direct usage of external functions that modify memory
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
    "default lint description"
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
    "default lint description"
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
    "default lint description"
}

declare_lint_pass!(GuidelineLints => [
    MEM_UNSAFE_FUNCTIONS,
    UNTRUSTED_LIB_LOADING,
    PASSING_STRING_TO_C_FUNCTIONS,
    FALLIABLE_MEMORY_ALLOCATION,
]);

impl<'tcx> LateLintPass<'tcx> for GuidelineLints {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: intravisit::FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'_>,
        _body: &'tcx hir::Body<'_>,
        span: Span,
        _def_id: LocalDefId,
    ) {
        mem_unsafe_functions::check(cx, span);
    }

    fn check_item(&mut self, _cx: &LateContext<'tcx>, _item: &'tcx hir::Item<'_>) {}
}
