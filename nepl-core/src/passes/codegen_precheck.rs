extern crate alloc;

use alloc::vec::Vec;

use crate::codegen_wasm;
use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::{HirBlock, HirBody, HirExpr, HirExprKind, HirModule};
use crate::types::TypeCtx;

pub fn precheck_wasm_codegen(ctx: &TypeCtx, module: &HirModule) -> Vec<Diagnostic> {
    let mut out = Vec::new();

    for ext in &module.externs {
        if codegen_wasm::wasm_sig_ids(ctx, ext.result, &ext.params).is_none() {
            out.push(
                Diagnostic::error("unsupported extern signature for wasm", ext.span)
                    .with_id(DiagnosticId::CodegenWasmUnsupportedExternSignature),
            );
        }
    }

    let reachable_functions = codegen_wasm::collect_reachable_wasm_functions(module);
    for f in &module.functions {
        if !reachable_functions.contains(&f.name) {
            continue;
        }
        if codegen_wasm::wasm_sig(ctx, f.result, &f.params).is_none()
            && !codegen_wasm::should_skip_wasm_codegen_for_generic(ctx, f)
        {
            out.push(
                Diagnostic::error("unsupported function signature for wasm", f.span)
                    .with_id(DiagnosticId::CodegenWasmUnsupportedFunctionSignature),
            );
        }
        if !codegen_wasm::should_skip_wasm_codegen_for_generic(ctx, f) {
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
                precheck_wasm_indirect_signature(ctx, block, &mut out);
            }
            out.extend(codegen_wasm::precheck_raw_wasm_body(f));
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

fn precheck_wasm_indirect_signature(ctx: &TypeCtx, block: &HirBlock, out: &mut Vec<Diagnostic>) {
    for line in &block.lines {
        check_indirect_sig_expr(ctx, &line.expr, out);
    }
}

fn check_indirect_sig_expr(ctx: &TypeCtx, expr: &HirExpr, out: &mut Vec<Diagnostic>) {
    match &expr.kind {
        HirExprKind::CallIndirect {
            callee,
            params,
            result,
            args,
        } => {
            if codegen_wasm::wasm_sig_ids(ctx, *result, params).is_none() {
                out.push(
                    Diagnostic::error("unsupported indirect call signature for wasm", expr.span)
                        .with_id(DiagnosticId::CodegenWasmUnsupportedIndirectSignature),
                );
            }
            check_indirect_sig_expr(ctx, callee, out);
            for arg in args {
                check_indirect_sig_expr(ctx, arg, out);
            }
        }
        HirExprKind::Call { args, .. } | HirExprKind::Intrinsic { args, .. } => {
            for arg in args {
                check_indirect_sig_expr(ctx, arg, out);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            check_indirect_sig_expr(ctx, cond, out);
            check_indirect_sig_expr(ctx, then_branch, out);
            check_indirect_sig_expr(ctx, else_branch, out);
        }
        HirExprKind::While { cond, body } => {
            check_indirect_sig_expr(ctx, cond, out);
            check_indirect_sig_expr(ctx, body, out);
        }
        HirExprKind::Match { scrutinee, arms } => {
            check_indirect_sig_expr(ctx, scrutinee, out);
            for arm in arms {
                check_indirect_sig_expr(ctx, &arm.body, out);
            }
        }
        HirExprKind::Block(b) => precheck_wasm_indirect_signature(ctx, b, out),
        HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
            check_indirect_sig_expr(ctx, value, out);
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(p) = payload {
                check_indirect_sig_expr(ctx, p, out);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for f in fields {
                check_indirect_sig_expr(ctx, f, out);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for it in items {
                check_indirect_sig_expr(ctx, it, out);
            }
        }
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
            check_indirect_sig_expr(ctx, inner, out);
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
