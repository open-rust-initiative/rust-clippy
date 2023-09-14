use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::{path_res, peel_hir_expr_refs};
use rustc_data_structures::fx::FxHashMap;
use rustc_hir::def::Res;
use rustc_hir::hir_id::{HirId, HirIdSet};
use rustc_hir::intravisit::{walk_block, walk_expr, Visitor};
use rustc_hir::{Block, Expr, ExprKind, Local, Ty, TyKind};
use rustc_hir_analysis::hir_ty_to_ty;
use rustc_lint::LateContext;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

use super::peel_casts;
use super::RETURN_STACK_ADDRESS;

pub(super) fn check<'tcx>(cx: &LateContext<'tcx>, block: &'tcx Block<'tcx>, visited_blocks: &mut HirIdSet) {
    if visited_blocks.contains(&block.hir_id) {
        return;
    }

    let mut collector = LocalAndRetCollector {
        local: FxHashMap::default(),
        rets: Vec::new(),
        visited_blocks,
        tcx: cx.tcx,
    };
    collector.visit_block(block);

    for ret in collector.rets {
        match &ret.kind {
            ExprKind::Cast(
                cast_expr,
                Ty {
                    kind: TyKind::Ptr(_), ..
                },
                ..,
            ) => {
                let castee = peel_hir_expr_refs(peel_casts(cast_expr)).0;
                let maybe_path = peel_method_calls(castee);
                if let Some(decl_span) = get_value_from_local_set(cx, maybe_path, &collector.local) {
                    emit_lint(cx, ret.span, decl_span);
                }
            },
            ExprKind::MethodCall(_, caller, ..) => {
                let maybe_path = peel_method_calls(caller);
                if let Some(decl_span) = get_value_from_local_set(cx, maybe_path, &collector.local) {
                    emit_lint(cx, ret.span, decl_span);
                }
            },
            _ => (),
        }
    }
}

fn get_value_from_local_set(cx: &LateContext<'_>, maybe_path: &Expr<'_>, set: &FxHashMap<HirId, Span>) -> Option<Span> {
    if let Res::Local(hir_id) = path_res(cx, maybe_path) {
        set.get(&hir_id).copied()
    } else {
        None
    }
}

fn peel_method_calls<'a>(expr: &'a Expr<'a>) -> &'a Expr<'a> {
    if let ExprKind::MethodCall(_, caller, ..) = &expr.kind {
        peel_method_calls(caller)
    } else {
        expr
    }
}

struct LocalAndRetCollector<'a, 'tcx> {
    local: FxHashMap<HirId, Span>,
    rets: Vec<&'a Expr<'tcx>>,
    visited_blocks: &'a mut HirIdSet,
    tcx: TyCtxt<'tcx>,
}

impl<'a, 'v> Visitor<'v> for LocalAndRetCollector<'a, 'v> {
    fn visit_local(&mut self, local: &'v Local<'v>) {
        if matches!(
            local.init,
            Some(
                Expr {
                    kind: ExprKind::Lit(_),
                    ..
                },
                ..
            )
        ) || matches!(local.ty, Some(ty) if hir_ty_to_ty(self.tcx, ty).is_simple_ty())
        {
            self.local.insert(local.pat.hir_id, local.span);
        }
    }
    fn visit_expr(&mut self, expr: &'v Expr<'v>) {
        match &expr.kind {
            ExprKind::Assign(assign_to, assigning, _) if matches!(assigning.kind, ExprKind::Lit(_)) => {
                self.local.insert(assign_to.hir_id, expr.span);
            },
            ExprKind::Ret(Some(ret_expr)) => {
                self.rets.push(ret_expr);
            },
            _ => walk_expr(self, expr),
        }
    }
    fn visit_block(&mut self, block: &'v Block<'v>) {
        self.visited_blocks.insert(block.hir_id);
        if let Some(ret_expr) = block.expr {
            self.rets.push(ret_expr);
        }
        walk_block(self, block);
    }
}

fn emit_lint(cx: &LateContext<'_>, span: Span, decl_span: Span) {
    span_lint_and_then(
        cx,
        RETURN_STACK_ADDRESS,
        span,
        "returning a pointer to stack address",
        |diag| {
            diag.span_note(decl_span, "local variable declared/assigned here");
            diag.help("consider declaring it as const or static variable");
        },
    );
}
