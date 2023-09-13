use std::ops::ControlFlow;

use clippy_utils::consts::{constant, Constant};
use clippy_utils::visitors::for_each_expr;
use clippy_utils::{
    fn_def_id, is_integer_literal, is_lint_allowed, is_path_diagnostic_item, path_res, peel_hir_expr_while,
};
use rustc_hir::def::Res;
use rustc_hir::def_id::DefIdSet;
use rustc_hir::hir_id::HirId;
use rustc_hir::{Block, BorrowKind, Expr, ExprKind, Local, Node, TyKind, UnOp};
use rustc_lint::LateContext;
use rustc_span::symbol::sym;
use rustc_span::Span;

use if_chain::if_chain;

use super::DANGLING_PTR_DEREFERENCE;
use super::NULL_PTR_DEREFERENCE;
use super::PTR_DOUBLE_FREE;

macro_rules! span_ptr_lint {
    ($cx:expr, $lint:ident, $span:expr, $note_span:expr) => {{
        let msg_and_note_msg = match stringify!($lint) {
            "NULL_PTR_DEREFERENCE" => Some(("dereferencing null pointer", "first dereference occurred here")),
            "DANGLING_PTR_DEREFERENCE" => Some((
                "dereferencing a raw pointer that was already freed",
                "the pointer was freed here",
            )),
            "PTR_DOUBLE_FREE" => Some(("pointer was freed multiple times", "second free occurred here")),
            _ => None,
        };
        if let Some((msg, note_msg)) = msg_and_note_msg {
            clippy_utils::diagnostics::span_lint_and_note($cx, $lint, $span, msg, Some($note_span), note_msg);
        }
    }};
}

pub(super) fn check_assign<'tcx, 'a: 'tcx>(cx: &LateContext<'tcx>, expr: &'a Expr<'tcx>) {
    if_chain! {
        if !is_lint_allowed(cx, NULL_PTR_DEREFERENCE, expr.hir_id);
        if !expr.span.from_expansion();
        if let Some(mut ptr_validator) = PtrValidator::from_assignment(cx, expr);
        then {
            ptr_validator.visit(cx);
            if let Some(deref_span) = ptr_validator.deref_span {
                span_ptr_lint!(cx, NULL_PTR_DEREFERENCE, expr.span, deref_span);
            }
        }
    }
}

pub(super) fn check_local<'tcx, 'a: 'tcx>(cx: &LateContext<'tcx>, local: &'a Local<'tcx>) {
    if_chain! {
        if !is_lint_allowed(cx, NULL_PTR_DEREFERENCE, local.hir_id);
        if !local.span.from_expansion();
        if let Some(mut ptr_validator) = PtrValidator::from_init(cx, local);
        then {
            ptr_validator.visit(cx);
            if let Some(deref_span) = ptr_validator.deref_span {
                span_ptr_lint!(cx, NULL_PTR_DEREFERENCE, local.span, deref_span);
            }
        }
    }
}

pub(super) fn check_call(cx: &LateContext<'_>, call_expr: &Expr<'_>, free_fns: &DefIdSet) {
    if !is_lint_allowed(cx, DANGLING_PTR_DEREFERENCE, call_expr.hir_id)
        || !is_lint_allowed(cx, PTR_DOUBLE_FREE, call_expr.hir_id)
    {
        let Some(mut ptr_validator) = PtrValidator::from_call(cx, call_expr, free_fns) else { return };

        ptr_validator.visit(cx);

        if let Some(deref_span) = ptr_validator.deref_span {
            span_ptr_lint!(cx, DANGLING_PTR_DEREFERENCE, deref_span, call_expr.span);
        }
        if let Some(free_span) = ptr_validator.free_span {
            span_ptr_lint!(cx, PTR_DOUBLE_FREE, call_expr.span, free_span);
        }
    }
}

struct PtrValidator<'a, 'tcx> {
    /// HirId of this pointer variable
    hir_id: HirId,
    parent_block: &'a Block<'tcx>,
    search_span: Span,
    is_null: bool,
    /// This should be the span where this pointer gets dereferenced.
    deref_span: Option<Span>,
    /// True if this validator was initialized from a `free` function call.
    was_freed: bool,
    /// This should be the span where this pointer gets freed.
    free_span: Option<Span>,
    /// The function id set for deallocation functions,
    /// it's used for identifying where this pointer got freed.
    free_fns: Option<&'a DefIdSet>,
}

impl<'a, 'tcx> PtrValidator<'a, 'tcx> {
    /// From a `.. = ...` expression, `i.e.: x = null()`
    fn from_assignment(cx: &'a LateContext<'tcx>, expr: &'a Expr<'tcx>) -> Option<Self> {
        if let ExprKind::Assign(assign_to, assigning, _) = expr.kind &&
            let Res::Local(hir_id) = path_res(cx, assign_to)
        {
            let parent_block = get_parent_block(cx, expr.hir_id)?;
            let assign_null = expr_is_creating_null_ptr(cx, assigning);
            Some(Self::new(expr.span, hir_id, assign_null, parent_block))
        } else {
            None
        }
    }
    /// From a `let .. = ...` pattern, `i.e.: let x: *mut i8 = null()`
    fn from_init(cx: &'a LateContext<'tcx>, local: &'a Local<'tcx>) -> Option<Self> {
        if let Local {
            pat,
            init: Some(init_expr),
            ..
        } = local
        {
            let parent_block = get_parent_block(cx, local.hir_id)?;
            let declared_null = expr_is_creating_null_ptr(cx, init_expr);
            Some(Self::new(local.span, pat.hir_id, declared_null, parent_block))
        } else {
            None
        }
    }
    /// From a call to free functions, where the free functions are defined by user config
    fn from_call(cx: &'a LateContext<'tcx>, call_expr: &Expr<'_>, free_fns: &'a DefIdSet) -> Option<Self> {
        // FIXME: It might not always the case that the first parameter is the pointer got freed,
        // we need a more ergonomic solution.
        if let ExprKind::Call(_, [first_param, ..]) = call_expr.kind &&
            let Res::Local(hir_id) = path_res(cx, first_param)
        {
            let parent_block = get_parent_block(cx, call_expr.hir_id)?;
            let assumed_null = false;
            Some(
                Self::new(call_expr.span, hir_id, assumed_null, parent_block)
                    .was_freed(true)
                    .free_fns(free_fns)
            )
        } else {
            None
        }
    }

    fn new(span: Span, path_hir_id: HirId, is_null: bool, parent_block: &'a Block<'tcx>) -> Self {
        // This will produce:
        // { let ptr = std::ptr::null(); do_something(); }
        //   --------------------------: span
        // -----------------------------------------------: parent_block
        //                              ------------------: search_span
        let search_span = parent_block.span.with_lo(span.hi());
        Self {
            hir_id: path_hir_id,
            parent_block,
            search_span,
            is_null,
            deref_span: None,
            was_freed: false,
            free_span: None,
            free_fns: None,
        }
    }

    fn was_freed(mut self, yes: bool) -> Self {
        self.was_freed = yes;
        self
    }

    fn free_fns(mut self, ids: &'a DefIdSet) -> Self {
        self.free_fns = Some(ids);
        self
    }

    fn visit(&mut self, cx: &'a LateContext<'tcx>) {
        let _res: Option<()> = for_each_expr(self.parent_block, |ex| {
            // Skipping irrelevant span
            if !self.search_span.contains(ex.span) {
                return ControlFlow::Continue(());
            }
            match ex.kind {
                // check for `ptr = non_null()` and `*ptr = some_value()`
                ExprKind::Assign(assign_to, assigning, _) => {
                    if path_hir_id_matches_id(cx, assign_to, self.hir_id) {
                        self.is_null = expr_is_creating_null_ptr(cx, assigning);
                        self.was_freed = false;
                    }
                },
                ExprKind::Unary(UnOp::Deref, expr) => {
                    if path_hir_id_matches_id(cx, expr, self.hir_id) &&
                        (self.is_null || self.was_freed)
                    {
                        self.deref_span = Some(ex.span);
                        return ControlFlow::Break(());
                    }
                },
                // FIXME: It might not always the case that the first parameter is the pointer got freed,
                // we need a more ergonomic solution.
                ExprKind::Call(_ , [first_arg, ..]) => {
                    if path_hir_id_matches_id(cx, first_arg, self.hir_id) &&
                        let Some(free_fn_ids) = self.free_fns &&
                        let Some(fn_did) = fn_def_id(cx, ex) &&
                        free_fn_ids.contains(&fn_did) &&
                        self.was_freed
                    {
                        self.free_span = Some(ex.span);
                        return ControlFlow::Break(());
                    }
                },
                _ => {
                    if path_hir_id_matches_id(cx, ex, self.hir_id) {
                        self.is_null = false;
                    }
                },
            }
            ControlFlow::Continue(())
        });
    }
}

/// Checks the given expression to see if its a:
///
/// 1. `std::ptr::null` or `std::ptr::null_mut` call.
/// 2. Integer `0` casting as a raw pointer.
/// 3. Constant that can be resolved as null pointer.
fn expr_is_creating_null_ptr(cx: &LateContext<'_>, expr: &Expr<'_>) -> bool {
    match expr.kind {
        ExprKind::Path(_) if matches!(constant(cx, cx.typeck_results(), expr), Some((Constant::RawPtr(0), _))) => true,
        ExprKind::Cast(inner_expr, cast_ty)
            if is_integer_literal(peel_casts(inner_expr), 0) && matches!(cast_ty.kind, TyKind::Ptr(_)) =>
        {
            true
        },
        ExprKind::Call(call, [])
            if is_path_diagnostic_item(cx, call, sym::ptr_null)
                || is_path_diagnostic_item(cx, call, sym::ptr_null_mut) =>
        {
            true
        },
        _ => false,
    }
}

/// Peels all casts and return the inner most (non-cast) expression.
///
/// i.e.
///
/// ```
/// some_expr as *mut i8 as *mut i16 as *mut i32 as *mut i64
/// ```
///
/// Will return expression for `some_expr`.
fn peel_casts<'a, 'tcx>(maybe_cast_expr: &'a Expr<'tcx>) -> &'a Expr<'tcx> {
    if let ExprKind::Cast(expr, _) = maybe_cast_expr.kind {
        peel_casts(expr)
    } else {
        maybe_cast_expr
    }
}

// FIXME: this will have FN if the pointer was declared globally,
// i.e. `const PTR: *mut i8 = 0 as *mut _;`, which should be fine at the moment,
// since its hard to detect dereferecing on a global null pointer without the help of mir.
fn get_parent_block<'tcx, 'a>(cx: &LateContext<'tcx>, id: HirId) -> Option<&'a Block<'tcx>> {
    for (_, node) in cx.tcx.hir().parent_iter(id) {
        if let Node::Block(block) = node {
            return Some(block);
        }
    }
    None
}

fn path_hir_id_matches_id(cx: &LateContext<'_>, maybe_path: &Expr<'_>, id: HirId) -> bool {
    let peeled = peel_hir_expr_while(maybe_path, |e| match e.kind {
        ExprKind::AddrOf(BorrowKind::Ref, _, e) | ExprKind::Unary(_, e) => Some(e),
        _ => None,
    });

    matches!(path_res(cx, peeled), Res::Local(hid) if hid == id)
}
