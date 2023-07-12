use super::EXTERN_WITHOUT_REPR;
use clippy_utils::diagnostics::span_lint_and_note;
use clippy_utils::ty::walk_ptrs_hir_ty;
use rustc_hir::{FnHeader, FnSig, ForeignItemKind, Item, ItemKind, Node, Ty};
use rustc_hir_analysis::hir_ty_to_ty;
use rustc_lint::LateContext;
use rustc_span::Span;
use rustc_target::spec::abi::Abi;

pub(super) fn check_item<'tcx>(cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
    match &item.kind {
        ItemKind::Fn(
            FnSig {
                header: FnHeader { abi: Abi::C { .. }, .. },
                decl,
                ..
            },
            ..,
        ) => lint_for_tys(cx, decl.inputs),
        ItemKind::ForeignMod {
            abi: Abi::C { .. },
            items,
        } => {
            for f_item in items.iter() {
                if let Some(Node::ForeignItem(f)) = cx.tcx.hir().find(f_item.id.hir_id()) {
                    if let ForeignItemKind::Fn(decl, ..) = f.kind {
                        lint_for_tys(cx, decl.inputs);
                    }
                }
            }
        },
        _ => (),
    }
}

/// Return the span of where the given `ty` was declared if it DOES NOT
/// have `repr(C|transparent|packed|align(x))` attribute.
fn non_ffi_safe_ty_span(cx: &LateContext<'_>, ty: &Ty<'_>) -> Option<Span> {
    let mid_ty = hir_ty_to_ty(cx.tcx, walk_ptrs_hir_ty(ty));
    let adt = mid_ty.ty_adt_def()?;
    let repr = adt.repr();
    if repr.c() || repr.transparent() || repr.packed() || repr.align.is_some() {
        None
    } else {
        Some(cx.tcx.def_span(adt.did()))
    }
}

fn lint_for_tys(cx: &LateContext<'_>, tys: &[Ty<'_>]) {
    let decl_and_usage_spans: Vec<(Span, Span)> = tys
        .iter()
        .filter_map(|ty| non_ffi_safe_ty_span(cx, ty).map(|s| (s, ty.span)))
        .collect();
    for (decl_span, usage_span) in decl_and_usage_spans {
        span_lint_and_note(
            cx,
            EXTERN_WITHOUT_REPR,
            usage_span,
            "should use `#[repr(..)]` to specifing data layout when type is used in FFI",
            Some(decl_span),
            "type declared here",
        );
    }
}
