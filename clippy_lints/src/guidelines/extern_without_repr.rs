use rustc_hir::{ForeignItemKind, Item, ItemKind, Node};
use rustc_lint::LateContext;

use super::EXTERN_WITHOUT_REPR;
use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::source::snippet_with_applicability;
use clippy_utils::ty::walk_ptrs_hir_ty;
use if_chain::if_chain;
use rustc_errors::Applicability;
use rustc_hir_analysis::hir_ty_to_ty;
use rustc_target::spec::abi::Abi;

pub(super) fn check_item<'tcx>(cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
    let msg = "Should use repr to specifing data layout when struct is used in FFI";
    if let ItemKind::Fn(fn_sig, _, _) = &item.kind {
        let mut app = Applicability::MaybeIncorrect;
        let snippet = snippet_with_applicability(cx, fn_sig.span, "..", &mut app);
        if let Some((fn_attrs, _)) = snippet.split_once("fn") {
            if fn_attrs.contains("extern \"C\"") {
                for i in 0..fn_sig.decl.inputs.len() {
                    let t = hir_ty_to_ty(cx.tcx, walk_ptrs_hir_ty(&fn_sig.decl.inputs[i]));
                    if let Some(adt) = t.ty_adt_def() {
                        let repr = adt.repr();
                        if repr.packed() || repr.transparent() || repr.c() || repr.align.is_some() {
                            continue;
                        }
                        let struct_span = cx.tcx.def_span(adt.did());
                        span_lint_and_then(cx, EXTERN_WITHOUT_REPR, struct_span, msg, |_| {});
                    }
                }
            }
        }
    }

    if_chain! {
        if let ItemKind::ForeignMod { abi, items } = &item.kind;
        if let Abi::C { unwind: _ } = abi;
        then {
            for i in 0..items.len() {
                if let Some(Node::ForeignItem(f)) = cx.tcx.hir().find(items[i].id.hir_id()) {
                    if let ForeignItemKind::Fn(decl, ..) = f.kind {
                        for j in 0..decl.inputs.len() {
                            let t = hir_ty_to_ty(cx.tcx, walk_ptrs_hir_ty(&decl.inputs[j]));
                            if let Some(adt) = t.ty_adt_def() {
                                let repr = adt.repr();
                                if repr.packed()
                                    || repr.transparent()
                                    || repr.c()
                                    || repr.simd()
                                    || repr.align.is_some()
                                {
                                    continue;
                                }
                                let struct_span = cx.tcx.def_span(adt.did());
                                span_lint_and_then(cx, EXTERN_WITHOUT_REPR, struct_span, msg, |_| {});
                            }
                        }
                    }
                }
            }
        }
    }
}
