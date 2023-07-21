use clippy_utils::diagnostics::span_lint_and_help;
use clippy_utils::source::snippet_opt;
use if_chain::if_chain;
use rustc_data_structures::fx::FxHashSet;
use rustc_hir::{Expr, Item, ItemKind, OwnerNode};
use rustc_lint::LateContext;
use rustc_span::hygiene::{ExpnKind, MacroKind};
use rustc_span::{sym, symbol::Ident, Span};

// FIXME: Currently, this lint does not have a working ui test,
// because it needs to be tested with `#[proc_macro]` attribute,
// but I couldn't find information about how to write such ui test in Clippy.
// So, right now the only solution was to compile first, then test in real code,
// which is a big headache.
use super::UNSAFE_BLOCK_IN_PROC_MACRO;

pub(super) fn check(cx: &LateContext<'_>, expr: &Expr<'_>, call_sites: &mut FxHashSet<Span>) {
    let expn_data = expr.span.ctxt().outer_expn_data();
    let call_site = expn_data.call_site;

    if_chain! {
        if !call_site.from_expansion();
        if call_sites.insert(call_site);
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
    let hir_map = cx.tcx.hir();
    let mut owner_ident: Option<Ident> = None;
    for (hid, node) in hir_map.parent_owner_iter(expr.hir_id) {
        if let OwnerNode::Item(Item {
            kind: ItemKind::Fn(..), ..
        }) = node
        {
            let attrs = hir_map.attrs(hid.into());
            if attrs
                .iter()
                .any(|attr| matches!(attr.ident(), Some(ident) if ident.name == sym::proc_macro))
            {
                owner_ident = node.ident();
                break;
            }
        }
    }
    let Some(ident) = owner_ident else { return };
    let name = ident.name.as_str();
    let msg = &format!("function-like procedural macro `{name}` could masking unsafe operations");
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
