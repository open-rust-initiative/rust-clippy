use clippy_utils::diagnostics::span_lint_and_help;
use if_chain::if_chain;
use rustc_hir::def::Res;
use rustc_hir::def_id::DefIdSet;
use rustc_hir::{Expr, ExprKind, Item, ItemKind, QPath};
use rustc_lint::LateContext;

use super::MEM_UNSAFE_FUNCTIONS;

/// Check extern function definitions.
///
/// The main purpose of this function is to load `def_ids` of declared external functions.
pub(super) fn check_foreign_item(item: &Item<'_>, blacklist: &[String], blacklist_ids: &mut DefIdSet) {
    if let ItemKind::ForeignMod { items, .. } = item.kind {
        for f_item in items {
            if blacklist.contains(&f_item.ident.as_str().to_string()) {
                let f_did = f_item.id.hir_id().owner.def_id.to_def_id();
                blacklist_ids.insert(f_did);
            }
        }
    }
}

/// Check function call expression
///
/// Will lint if the name of called function was blacklisted by the configuration.
pub(super) fn check(cx: &LateContext<'_>, expr: &Expr<'_>, blacklist_ids: &DefIdSet) {
    if_chain! {
        if let ExprKind::Call(fn_expr, _) = &expr.kind;
        if let ExprKind::Path(qpath) = &fn_expr.kind;
        if let QPath::Resolved(_, path) = qpath;
        if let Res::Def(_, did) = path.res;
        if blacklist_ids.contains(&did);
        then {
            span_lint_and_help(
                cx,
                MEM_UNSAFE_FUNCTIONS,
                fn_expr.span,
                "use of potentially dangerous memory manipulation function",
                None,
                "consider using its safe version",
            );
        }
    }
}
