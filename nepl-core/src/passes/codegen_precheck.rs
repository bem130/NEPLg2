extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::{HirBlock, HirBody, HirExpr, HirExprKind, HirModule};
use crate::types::TypeCtx;
use crate::wasm_shared;
use wasm_encoder::ValType;

type WasmSig = (Vec<ValType>, Vec<ValType>);

pub fn precheck_wasm_codegen(ctx: &TypeCtx, module: &HirModule) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let wasm_sig_set = wasm_shared::collect_wasm_signature_set(ctx, module);

    for ext in &module.externs {
        if wasm_shared::wasm_sig_ids(ctx, ext.result, &ext.params).is_none() {
            out.push(
                Diagnostic::error("unsupported extern signature for wasm", ext.span)
                    .with_id(DiagnosticId::CodegenWasmUnsupportedExternSignature),
            );
        }
    }

    let reachable_functions = wasm_shared::collect_reachable_wasm_functions(module);
    for f in &module.functions {
        if !reachable_functions.contains(&f.name) {
            continue;
        }
        if wasm_shared::wasm_sig(ctx, f.result, &f.params).is_none()
            && !wasm_shared::should_skip_wasm_codegen_for_generic(ctx, f)
        {
            out.push(
                Diagnostic::error("unsupported function signature for wasm", f.span)
                    .with_id(DiagnosticId::CodegenWasmUnsupportedFunctionSignature),
            );
        }
        if !wasm_shared::should_skip_wasm_codegen_for_generic(ctx, f) {
            let result_kind = ctx.get(ctx.resolve_id(f.result));
            if !matches!(result_kind, crate::types::TypeKind::Unit) {
                if let HirBody::Block(block) = &f.body {
                    if !block_produces_value(ctx, block) {
                        out.push(
                            Diagnostic::error("function expected to return value", f.span)
                                .with_id(DiagnosticId::CodegenWasmMissingReturnValue),
                        );
                    }
                }
            }
            if matches!(f.body, HirBody::LlvmIr(_)) {
                out.push(
                    Diagnostic::error("llvm ir block cannot be compiled by wasm backend", f.span)
                        .with_id(DiagnosticId::CodegenWasmLlvmIrBodyNotSupported),
                );
            }
            if let HirBody::Block(block) = &f.body {
                precheck_wasm_indirect_signature(ctx, block, &wasm_sig_set, &mut out);
            }
            out.extend(wasm_shared::precheck_raw_wasm_body(ctx, f));
        }
    }

    out
}

fn block_produces_value(ctx: &TypeCtx, block: &HirBlock) -> bool {
    let mut last_non_drop_line_ty_is_value = false;
    for line in &block.lines {
        if line.drop_result {
            continue;
        }
        let ty = ctx.get(ctx.resolve_id(line.expr.ty));
        last_non_drop_line_ty_is_value = !matches!(ty, crate::types::TypeKind::Unit);
    }
    last_non_drop_line_ty_is_value
}

fn precheck_wasm_indirect_signature(
    ctx: &TypeCtx,
    block: &HirBlock,
    wasm_sig_set: &BTreeSet<WasmSig>,
    out: &mut Vec<Diagnostic>,
) {
    for line in &block.lines {
        check_indirect_sig_expr(ctx, &line.expr, wasm_sig_set, out);
    }
}

fn check_indirect_sig_expr(
    ctx: &TypeCtx,
    expr: &HirExpr,
    wasm_sig_set: &BTreeSet<WasmSig>,
    out: &mut Vec<Diagnostic>,
) {
    match &expr.kind {
        HirExprKind::CallIndirect {
            callee,
            params,
            result,
            args,
        } => {
            if let Some(sig) = wasm_shared::wasm_sig_ids(ctx, *result, params) {
                if !wasm_sig_set.contains(&sig) {
                    out.push(
                        Diagnostic::error("missing wasm signature for indirect call", expr.span)
                            .with_id(DiagnosticId::CodegenWasmMissingIndirectSignature),
                    );
                }
            } else {
                out.push(
                    Diagnostic::error("unsupported indirect call signature for wasm", expr.span)
                        .with_id(DiagnosticId::CodegenWasmUnsupportedIndirectSignature),
                );
            }
            check_indirect_sig_expr(ctx, callee, wasm_sig_set, out);
            for arg in args {
                check_indirect_sig_expr(ctx, arg, wasm_sig_set, out);
            }
        }
        HirExprKind::Call { args, .. } => {
            for arg in args {
                check_indirect_sig_expr(ctx, arg, wasm_sig_set, out);
            }
        }
        HirExprKind::Intrinsic { name, args, .. } => {
            if !wasm_shared::is_supported_wasm_intrinsic(name) {
                out.push(
                    Diagnostic::error("unknown codegen intrinsic", expr.span)
                        .with_id(DiagnosticId::CodegenWasmUnknownIntrinsic),
                );
            }
            for arg in args {
                check_indirect_sig_expr(ctx, arg, wasm_sig_set, out);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            check_indirect_sig_expr(ctx, cond, wasm_sig_set, out);
            check_indirect_sig_expr(ctx, then_branch, wasm_sig_set, out);
            check_indirect_sig_expr(ctx, else_branch, wasm_sig_set, out);
        }
        HirExprKind::While { cond, body } => {
            check_indirect_sig_expr(ctx, cond, wasm_sig_set, out);
            check_indirect_sig_expr(ctx, body, wasm_sig_set, out);
        }
        HirExprKind::Match { scrutinee, arms } => {
            check_indirect_sig_expr(ctx, scrutinee, wasm_sig_set, out);
            for arm in arms {
                check_indirect_sig_expr(ctx, &arm.body, wasm_sig_set, out);
            }
        }
        HirExprKind::Block(b) => precheck_wasm_indirect_signature(ctx, b, wasm_sig_set, out),
        HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
            check_indirect_sig_expr(ctx, value, wasm_sig_set, out);
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(p) = payload {
                check_indirect_sig_expr(ctx, p, wasm_sig_set, out);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for f in fields {
                check_indirect_sig_expr(ctx, f, wasm_sig_set, out);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for it in items {
                check_indirect_sig_expr(ctx, it, wasm_sig_set, out);
            }
        }
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
            check_indirect_sig_expr(ctx, inner, wasm_sig_set, out);
        }
        HirExprKind::Drop { .. } => {}
        HirExprKind::Unit
        | HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Var(_)
        | HirExprKind::FnValue(_) => {}
    }
}
