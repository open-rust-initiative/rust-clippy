use clippy_utils::diagnostics::span_lint_and_note;
use clippy_utils::{path_res, peel_hir_expr_refs};
use rustc_data_structures::fx::FxHashMap;
use rustc_hir::def::Res;
use rustc_hir::hir_id::{HirId, HirIdSet};
use rustc_hir::intravisit::{walk_block, walk_expr, Visitor};
use rustc_hir::{Block, Expr, ExprKind, Local, Path, QPath, Ty, TyKind};
use rustc_lint::LateContext;
use rustc_span::symbol::sym;
use rustc_span::Span;

use super::{peel_casts, RETURN_STACK_ADDRESS};

pub(super) fn check<'tcx>(cx: &LateContext<'tcx>, block: &'tcx Block<'tcx>, visited_blocks: &mut HirIdSet) {
    if visited_blocks.contains(&block.hir_id) {
        return;
    }

    let mut collector = LocalAndRetCollector {
        local: FxHashMap::default(),
        rets: Vec::new(),
        visited_blocks,
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
                if let Some(decl_span) = get_simple_local_ty_span(cx, maybe_path, &collector.local) {
                    emit_lint(cx, ret.span, decl_span);
                }
            },
            ExprKind::MethodCall(methond_name, caller, ..)
                if methond_name.ident.name == sym::as_ptr || methond_name.ident.name == sym::as_mut_ptr =>
            {
                let maybe_path = peel_method_calls(caller);
                if let Some(decl_span) = get_simple_local_ty_span(cx, maybe_path, &collector.local) {
                    emit_lint(cx, ret.span, decl_span);
                }
            },
            _ => (),
        }
    }
}

fn get_simple_local_ty_span(cx: &LateContext<'_>, maybe_path: &Expr<'_>, set: &FxHashMap<HirId, Span>) -> Option<Span> {
    if let Res::Local(hir_id) = path_res(cx, maybe_path) &&
        let Some(span) = set.get(&hir_id) &&
        cx.typeck_results().node_type(hir_id).is_simple_ty()
    {
        Some(*span)
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
}

impl<'a, 'v> Visitor<'v> for LocalAndRetCollector<'a, 'v> {
    fn visit_local(&mut self, local: &'v Local<'v>) {
        if local.ty.is_some()
            || matches!(
                local.init,
                Some(
                    Expr {
                        kind: ExprKind::Lit(_),
                        ..
                    },
                    ..
                )
            )
        {
            self.local.insert(local.pat.hir_id, local.span);
        }
    }
    fn visit_expr(&mut self, expr: &'v Expr<'v>) {
        match &expr.kind {
            ExprKind::Assign(assign_to, assigning, _) if matches!(assigning.kind, ExprKind::Lit(_)) => {
                if let ExprKind::Path(QPath::Resolved(
                    _,
                    Path {
                        res: Res::Local(hir_id),
                        ..
                    },
                )) = &assign_to.kind
                {
                    self.local.insert(*hir_id, expr.span);
                }
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
    span_lint_and_note(
        cx,
        RETURN_STACK_ADDRESS,
        span,
        "returning a pointer to stack address",
        Some(decl_span),
        "local variable declared/assigned here",
    );
}
