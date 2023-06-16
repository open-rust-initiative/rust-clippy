use clippy_utils::diagnostics::span_lint;
use rustc_lint::LateContext;
use rustc_span::Span;

use super::MEM_UNSAFE_FUNCTIONS;

// TODO: Adjust the parameters as necessary
pub(super) fn check(cx: &LateContext<'_>, span: Span) {
    span_lint(cx, MEM_UNSAFE_FUNCTIONS, span, "it's working!");
}
