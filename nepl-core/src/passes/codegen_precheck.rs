extern crate alloc;

use alloc::vec::Vec;

use crate::codegen_wasm;
use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::HirModule;
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
    }

    out
}
