//! LLVM IR 生成（core 側）
//!
//! このモジュールは AST から LLVM IR テキストを生成する責務のみを持つ。
//! clang 実行などのホスト依存処理は `nepl-cli` 側で扱う。

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::ast::{Block, FnBody, Ident, Literal, Module, PrefixExpr, PrefixItem, Stmt, TypeExpr};
use crate::compiler::{BuildProfile, CompileTarget};
use crate::ast::Directive;
use crate::hir::{FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirModule};
use crate::types::{TypeCtx, TypeId, TypeKind};

/// LLVM IR 生成時のエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlvmCodegenError {
    MissingLlvmIrBlock,
    UnsupportedParsedFunctionBody { function: String },
    UnsupportedWasmBody { function: String },
    ConflictingRawBodies { function: String },
    TypecheckFailed { reason: String },
    MissingEntryFunction { function: String },
    UnsupportedHirLowering { function: String, reason: String },
}

impl core::fmt::Display for LlvmCodegenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LlvmCodegenError::MissingLlvmIrBlock => {
                write!(
                    f,
                    "llvm target requires at least one #llvmir block in module/function body"
                )
            }
            LlvmCodegenError::UnsupportedParsedFunctionBody { function } => write!(
                f,
                "llvm target currently supports only subset lowering for parsed functions; function '{}' is not in supported subset",
                function
            ),
            LlvmCodegenError::UnsupportedWasmBody { function } => write!(
                f,
                "llvm target cannot lower #wasm function body; function '{}'",
                function
            ),
            LlvmCodegenError::ConflictingRawBodies { function } => write!(
                f,
                "function '{}' has multiple active raw bodies after #if gate evaluation",
                function
            ),
            LlvmCodegenError::TypecheckFailed { reason } => {
                write!(f, "failed to typecheck module for llvm lowering: {}", reason)
            }
            LlvmCodegenError::MissingEntryFunction { function } => write!(
                f,
                "entry function '{}' was not found in lowered module",
                function
            ),
            LlvmCodegenError::UnsupportedHirLowering { function, reason } => write!(
                f,
                "failed to lower function '{}' to llvm: {}",
                function,
                reason
            ),
        }
    }
}

enum RawBodySelection<'a> {
    None,
    Llvm(&'a crate::ast::LlvmIrBlock),
    Wasm,
    Conflict,
}

/// `#llvmir` ブロックを連結して LLVM IR テキストを生成する。
///
/// 現段階では手書き `#llvmir` を主経路とし、Parsed 関数は最小 subset のみ lower する。
pub fn emit_ll_from_module(module: &Module) -> Result<String, LlvmCodegenError> {
    emit_ll_from_module_for_target(module, CompileTarget::Llvm, BuildProfile::Debug)
}

/// `target/profile` 条件を評価しながら LLVM IR を生成する。
pub fn emit_ll_from_module_for_target(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> Result<String, LlvmCodegenError> {
    validate_target_directive_for_llvm(module)?;
    let mut out = String::new();
    let entry_names = collect_active_entry_names(module, target, profile);
    let reachable_hint = compute_reachable_hint(module, target, profile, &entry_names);
    let mut emitted_functions: Vec<String> = Vec::new();
    let mut pending_if: Option<bool> = None;

    for stmt in &module.root.items {
        if let Stmt::Directive(d) = stmt {
            if let Some(allowed) = gate_allows(d, target, profile) {
                pending_if = Some(allowed);
                continue;
            }
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        if !allowed {
            continue;
        }

        match stmt {
            Stmt::LlvmIr(block) => {
                collect_defined_functions_from_llvmir_block(block, &mut emitted_functions);
                append_llvmir_block(&mut out, block);
            }
            Stmt::FnDef(def) => match &def.body {
                FnBody::LlvmIr(block) => {
                    if !is_ast_fn_reachable(def.name.name.as_str(), reachable_hint.as_ref()) {
                        continue;
                    }
                    collect_defined_functions_from_llvmir_block(block, &mut emitted_functions);
                    append_llvmir_block(&mut out, block);
                }
                FnBody::Parsed(block) => {
                    if !is_ast_fn_reachable(def.name.name.as_str(), reachable_hint.as_ref()) {
                        continue;
                    }
                    match select_raw_body_from_parsed_block(block, target, profile) {
                        RawBodySelection::Llvm(raw) => {
                            collect_defined_functions_from_llvmir_block(raw, &mut emitted_functions);
                            append_llvmir_block(&mut out, raw);
                        }
                        RawBodySelection::Wasm => {
                            return Err(LlvmCodegenError::UnsupportedWasmBody {
                                function: def.name.name.clone(),
                            });
                        }
                        RawBodySelection::Conflict => {
                            return Err(LlvmCodegenError::ConflictingRawBodies {
                                function: def.name.name.clone(),
                            });
                        }
                        RawBodySelection::None => {
                            if let Some(lowered) = lower_parsed_fn_with_gates(
                                def.name.name.as_str(),
                                &def.signature,
                                &def.params,
                                block,
                                target,
                                profile,
                            ) {
                                emitted_functions.push(def.name.name.clone());
                                out.push_str(&lowered);
                                out.push('\n');
                            }
                        }
                    }
                }
                FnBody::Wasm(_) => {
                    // `#wasm` は明示的な wasm backend 専用実装。
                    // 非 entry 関数は移行期間のためスキップするが、
                    // entry が #wasm のみの場合は LLVM 実行可能なモジュールを作れないためエラーとする。
                    if is_ast_fn_reachable(def.name.name.as_str(), reachable_hint.as_ref()) {
                        return Err(LlvmCodegenError::UnsupportedWasmBody {
                            function: def.name.name.clone(),
                        });
                    }
                }
            },
            _ => {}
        }
    }

    if let Some(entry) = entry_names.last() {
        let mut resolved_entry = entry.clone();
        if !emitted_functions.iter().any(|n| n == entry) {
            resolved_entry = try_lower_entry_from_hir(
                module,
                target,
                profile,
                entry.as_str(),
                &mut out,
                &mut emitted_functions,
            )?;
        }
        if !emitted_functions.iter().any(|n| n == &resolved_entry) {
            return Err(LlvmCodegenError::MissingEntryFunction {
                function: resolved_entry.clone(),
            });
        }
        if emitted_functions.iter().any(|n| n == &resolved_entry)
            && resolved_entry != "main"
            && !emitted_functions.iter().any(|n| n == "main")
        {
            out.push_str(&format!(
                "define i32 @main() {{\nentry:\n  %0 = call i32 @{}()\n  ret i32 %0\n}}\n\n",
                resolved_entry
            ));
        }
    }

    Ok(out)
}

fn compute_reachable_hint(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
    entry_names: &[String],
) -> Option<BTreeSet<String>> {
    if entry_names.is_empty() {
        return None;
    }
    let (_, hir) = try_build_hir_with_target(module, target, profile).ok()?;
    let mut out = BTreeSet::new();
    for entry in entry_names {
        let reachable = collect_reachable_functions(&hir, entry.as_str());
        for name in reachable {
            out.insert(name.clone());
            if let Some(sep) = find_mangled_signature_separator(name.as_str()) {
                out.insert(String::from(&name[..sep]));
            }
        }
    }
    Some(out)
}

fn is_ast_fn_reachable(name: &str, reachable_hint: Option<&BTreeSet<String>>) -> bool {
    match reachable_hint {
        None => true,
        Some(set) => set.contains(name),
    }
}

fn validate_target_directive_for_llvm(module: &Module) -> Result<(), LlvmCodegenError> {
    let mut found = false;
    for d in &module.directives {
        if let Directive::Target { target, .. } = d {
            if !is_known_target_name(target.as_str()) {
                return Err(LlvmCodegenError::TypecheckFailed {
                    reason: format!("unknown target in #target: {}", target),
                });
            }
            if found {
                return Err(LlvmCodegenError::TypecheckFailed {
                    reason: String::from("multiple #target directives are not allowed"),
                });
            }
            found = true;
        }
    }
    if !found {
        for stmt in &module.root.items {
            if let Stmt::Directive(Directive::Target { target, .. }) = stmt {
                if !is_known_target_name(target.as_str()) {
                    return Err(LlvmCodegenError::TypecheckFailed {
                        reason: format!("unknown target in #target: {}", target),
                    });
                }
                if found {
                    return Err(LlvmCodegenError::TypecheckFailed {
                        reason: String::from("multiple #target directives are not allowed"),
                    });
                }
                found = true;
            }
        }
    }
    Ok(())
}

fn is_known_target_name(name: &str) -> bool {
    matches!(name, "wasm" | "core" | "wasi" | "std" | "llvm")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LlTy {
    Void,
    I32,
    I64,
    F32,
    F64,
}

impl LlTy {
    fn ir(self) -> &'static str {
        match self {
            LlTy::Void => "void",
            LlTy::I32 => "i32",
            LlTy::I64 => "i64",
            LlTy::F32 => "float",
            LlTy::F64 => "double",
        }
    }
}

#[derive(Debug, Clone)]
struct FnSig {
    params: Vec<LlTy>,
    ret: LlTy,
}

#[derive(Debug, Clone)]
struct LocalBinding {
    ptr: String,
    ty: LlTy,
}

#[derive(Debug, Clone)]
struct LlValue {
    ty: LlTy,
    repr: String,
}

struct LowerCtx<'a> {
    function_name: &'a str,
    sigs: &'a BTreeMap<String, FnSig>,
    function_ids: BTreeMap<String, i32>,
    reachable: &'a BTreeSet<String>,
    strings: &'a [String],
    memory_global: &'a str,
    fallback_alloc_symbol: Option<&'a str>,
    out: String,
    tmp_seq: usize,
    label_seq: usize,
    scopes: Vec<BTreeMap<String, LocalBinding>>,
}

impl<'a> LowerCtx<'a> {
    fn new(
        function_name: &'a str,
        sigs: &'a BTreeMap<String, FnSig>,
        reachable: &'a BTreeSet<String>,
        strings: &'a [String],
        memory_global: &'a str,
        fallback_alloc_symbol: Option<&'a str>,
    ) -> Self {
        let mut function_ids = BTreeMap::new();
        let mut next_id = 1i32;
        for name in sigs.keys() {
            function_ids.insert(name.clone(), next_id);
            next_id += 1;
        }
        Self {
            function_name,
            sigs,
            function_ids,
            reachable,
            strings,
            memory_global,
            fallback_alloc_symbol,
            out: String::new(),
            tmp_seq: 0,
            label_seq: 0,
            scopes: Vec::new(),
        }
    }

    fn push_line(&mut self, line: &str) {
        self.out.push_str(line);
        self.out.push('\n');
    }

    fn next_tmp(&mut self) -> String {
        let name = format!("%t{}", self.tmp_seq);
        self.tmp_seq += 1;
        name
    }

    fn next_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_seq);
        self.label_seq += 1;
        label
    }

    fn begin_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn bind_local(&mut self, name: &str, ptr: String, ty: LlTy) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), LocalBinding { ptr, ty });
        }
    }

    fn lookup_local_current(&self, name: &str) -> Option<&LocalBinding> {
        self.scopes.last().and_then(|scope| scope.get(name))
    }

    fn lookup_local(&self, name: &str) -> Option<&LocalBinding> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v);
            }
        }
        None
    }

    fn lookup_local_fuzzy(&self, name: &str) -> Option<&LocalBinding> {
        if let Some(v) = self.lookup_local(name) {
            return Some(v);
        }
        if let Some((base, _)) = name.split_once("__") {
            return self.lookup_local(base);
        }
        None
    }

    fn function_id_of(&self, name: &str) -> Option<i32> {
        self.function_ids.get(name).copied()
    }

    fn linear_i8_ptr_from_i32(&mut self, offset_i32: &str) -> String {
        let idx_i64 = self.next_tmp();
        let ptr_i8 = self.next_tmp();
        self.push_line(&format!("  {} = zext i32 {} to i64", idx_i64, offset_i32));
        self.push_line(&format!(
            "  {} = getelementptr [67108864 x i8], [67108864 x i8]* {}, i64 0, i64 {}",
            ptr_i8, self.memory_global, idx_i64
        ));
        ptr_i8
    }

    fn linear_typed_ptr_from_i32(&mut self, offset_i32: &str, ty: LlTy) -> String {
        let base_i8 = self.linear_i8_ptr_from_i32(offset_i32);
        let typed_ptr = self.next_tmp();
        self.push_line(&format!(
            "  {} = bitcast i8* {} to {}*",
            typed_ptr,
            base_i8,
            ty.ir()
        ));
        typed_ptr
    }
}

fn try_lower_entry_from_hir(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
    entry: &str,
    out: &mut String,
    emitted_functions: &mut Vec<String>,
) -> Result<String, LlvmCodegenError> {
    let (mut types, mut hir) = build_hir_for_llvm_lowering(module, target, profile)?;
    crate::passes::insert_drops(&mut hir, types.unit());

    let mut function_map: BTreeMap<String, &HirFunction> = BTreeMap::new();
    for f in &hir.functions {
        function_map.insert(f.name.clone(), f);
    }
    let resolved_entry = if function_map.contains_key(entry) {
        String::from(entry)
    } else if let Some(found) = function_map
        .keys()
        .find(|name| {
            name.starts_with(&format!("{}__", entry))
                || name.starts_with(&format!("{}::", entry))
                || name.ends_with(&format!("::{}", entry))
        })
        .cloned()
    {
        found
    } else {
        let mut sample = function_map.keys().take(6).cloned().collect::<Vec<_>>();
        if sample.is_empty() {
            sample.push(String::from("<none>"));
        }
        return Err(LlvmCodegenError::MissingEntryFunction {
            function: format!("{} (available: {})", entry, sample.join(", ")),
        });
    };

    let mut sigs = collect_hir_signatures(&types, &hir);
    let mut reachable = collect_reachable_functions(&hir, resolved_entry.as_str());
    extend_reachable_with_runtime_helpers(&mut reachable, &hir, &sigs);
    let reachable_set: BTreeSet<String> = reachable.iter().cloned().collect();
    let fallback_alloc_symbol = resolve_symbol_name(&sigs, "alloc", &[LlTy::I32], LlTy::I32)
        .is_none()
        .then_some("__nepl_fallback_alloc");
    let memory_global = if fallback_alloc_symbol.is_some() {
        emit_fallback_linear_memory_runtime(out);
        "@__nepl_fallback_mem"
    } else {
        "@__nepl_mem"
    };

    let mut declared_extern_symbols: BTreeSet<String> = BTreeSet::new();
    for ex in &hir.externs {
        let local_name_raw = ex.local_name.as_str();
        let base_alias = find_mangled_signature_separator(local_name_raw)
            .map(|sep| &local_name_raw[..sep]);
        let needs_base = base_alias
            .map(|base| reachable.iter().any(|n| n == base))
            .unwrap_or(false);
        let local_name = ll_symbol(ex.local_name.as_str());
        let external_name = ll_symbol(ex.name.as_str());
        let params = ex
            .params
            .iter()
            .map(|t| llty_for_type(&types, *t).ir())
            .collect::<Vec<_>>()
            .join(", ");
        let ret = llty_for_type(&types, ex.result).ir();
        if declared_extern_symbols.insert(ex.name.clone()) {
            out.push_str(&format!("declare {} {}({})\n", ret, external_name, params));
        }
        if ex.local_name != ex.name {
            let args = ex
                .params
                .iter()
                .enumerate()
                .map(|(i, t)| format!("{} %a{}", llty_for_type(&types, *t).ir(), i))
                .collect::<Vec<_>>()
                .join(", ");
            let call_args = ex
                .params
                .iter()
                .enumerate()
                .map(|(i, t)| format!("{} %a{}", llty_for_type(&types, *t).ir(), i))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("define {} {}({}) {{\n", ret, local_name, args));
            out.push_str("entry:\n");
            if ret == "void" {
                out.push_str(&format!("  call {} {}({})\n", ret, external_name, call_args));
                out.push_str("  ret void\n");
            } else {
                out.push_str(&format!(
                    "  %ret = call {} {}({})\n",
                    ret, external_name, call_args
                ));
                out.push_str(&format!("  ret {} %ret\n", ret));
            }
            out.push_str("}\n");
        }
        if !emitted_functions.iter().any(|n| n == &ex.local_name) {
            emitted_functions.push(ex.local_name.clone());
        }
        if needs_base {
            if let Some(base) = base_alias {
                if base != ex.local_name
                    && !llvm_output_has_function(out, base)
                    && !emitted_functions.iter().any(|n| n == base)
                {
                    let args = ex
                        .params
                        .iter()
                        .enumerate()
                        .map(|(i, t)| format!("{} %a{}", llty_for_type(&types, *t).ir(), i))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let call_args = ex
                        .params
                        .iter()
                        .enumerate()
                        .map(|(i, t)| format!("{} %a{}", llty_for_type(&types, *t).ir(), i))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let base_sym = ll_symbol(base);
                    out.push_str(&format!("define {} {}({}) {{\n", ret, base_sym, args));
                    out.push_str("entry:\n");
                    if ret == "void" {
                        out.push_str(&format!("  call {} {}({})\n", ret, local_name, call_args));
                        out.push_str("  ret void\n");
                    } else {
                        out.push_str(&format!("  %ret = call {} {}({})\n", ret, local_name, call_args));
                        out.push_str(&format!("  ret {} %ret\n", ret));
                    }
                    out.push_str("}\n");
                    emitted_functions.push(String::from(base));
                }
            }
        }
    }
    if !reachable.is_empty() {
        out.push('\n');
    }

    for name in &reachable {
        if emitted_functions.iter().any(|n| n == name) {
            continue;
        }
        let Some(func) = function_map.get(name.as_str()) else {
            continue;
        };
        match &func.body {
            HirBody::LlvmIr(raw) => {
                out.push_str(&format!("; nepl: function {} (raw llvmir)\n", name));
                let mut defined = Vec::new();
                collect_defined_functions_from_llvmir_block(raw, &mut defined);
                let defines_current_name = defined.iter().any(|d| d == name);
                let already_defined = !defined.is_empty()
                    && defined.iter().all(|n| {
                        emitted_functions.iter().any(|e| e == n)
                            || llvm_output_has_function(out, n.as_str())
                    });
                if already_defined {
                    if defines_current_name && !emitted_functions.iter().any(|n| n == name) {
                        emitted_functions.push(name.clone());
                    }
                    continue;
                }
                append_llvmir_block(out, raw);
                for def in defined {
                    if !emitted_functions.iter().any(|n| n == &def) {
                        emitted_functions.push(def);
                    }
                }
                if !defines_current_name {
                    if let Some(sig) = sigs.get(name.as_str()) {
                        if let Some(sep) = find_mangled_signature_separator(name.as_str()) {
                            let base_name = &name[..sep];
                            let _ = emit_alias_to_symbol(
                                name.as_str(),
                                base_name,
                                sig,
                                out,
                                emitted_functions,
                            );
                        }
                    }
                }
                if defines_current_name && !emitted_functions.iter().any(|n| n == name) {
                    emitted_functions.push(name.clone());
                }
            }
            HirBody::Wasm(_) => {
                return Err(LlvmCodegenError::UnsupportedWasmBody {
                    function: func.name.clone(),
                });
            }
            HirBody::Block(block) => {
                out.push_str(&format!("; nepl: function {} (lowered block)\n", name));
                let lowered = lower_hir_function(
                    &types,
                    &hir,
                    &sigs,
                    &reachable_set,
                    memory_global,
                    fallback_alloc_symbol,
                    func,
                    block,
                )?;
                out.push_str(&lowered);
                out.push('\n');
                emitted_functions.push(name.clone());
            }
        }
    }

    let emitted_snapshot = emitted_functions.clone();
    for name in emitted_snapshot {
        let Some(sig) = sigs.get(name.as_str()) else {
            continue;
        };
        let _ = emit_base_alias_to_mangled(name.as_str(), sig, &sigs, out, emitted_functions);
    }

    for (name, sig) in sigs.iter() {
        if emitted_functions.iter().any(|n| n == name) {
            continue;
        }
        let _ = emit_base_alias_for_mangled(name.as_str(), sig, out, emitted_functions);
    }

    if resolved_entry == "main" && emitted_functions.iter().any(|n| n == "__nepl_entry_main") {
        out.push_str("define i32 @main() {\nentry:\n  call void @__nepl_entry_main()\n  ret i32 0\n}\n\n");
        emitted_functions.push(String::from("main"));
    }
    if llvm_output_mentions_symbol(out, "alloc") && !llvm_output_has_function(out, "alloc") {
        if !llvm_output_has_function(out, "__nepl_fallback_alloc") {
            emit_fallback_linear_memory_runtime(out);
        }
        out.push_str("define i32 @alloc(i32 %size) {\nentry:\n");
        out.push_str("  %0 = call i32 @__nepl_fallback_alloc(i32 %size)\n");
        out.push_str("  ret i32 %0\n}\n\n");
        if !emitted_functions.iter().any(|n| n == "alloc") {
            emitted_functions.push(String::from("alloc"));
        }
    }

    // suppress unused warning when future passes extend signature synthesis
    sigs.clear();
    Ok(resolved_entry)
}

fn emit_base_alias_for_mangled(
    mangled: &str,
    sig: &FnSig,
    out: &mut String,
    emitted_functions: &mut Vec<String>,
) -> bool {
    let Some(sep) = find_mangled_signature_separator(mangled) else {
        return false;
    };
    let base = &mangled[..sep];
    if base.is_empty() || base == mangled {
        return false;
    }
    emit_alias_to_symbol(mangled, base, sig, out, emitted_functions)
}

fn emit_alias_to_symbol(
    mangled: &str,
    base: &str,
    sig: &FnSig,
    out: &mut String,
    emitted_functions: &mut Vec<String>,
) -> bool {
    let base_available = emitted_functions.iter().any(|n| n == base) || llvm_output_has_function(out, base);
    if !base_available {
        return false;
    }
    if emitted_functions.iter().any(|n| n == mangled) || llvm_output_has_function(out, mangled) {
        return false;
    }
    let params = sig
        .params
        .iter()
        .enumerate()
        .map(|(i, ty)| format!("{} %p{}", ty.ir(), i))
        .collect::<Vec<_>>();
    let call_args = sig
        .params
        .iter()
        .enumerate()
        .map(|(i, ty)| format!("{} %p{}", ty.ir(), i))
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!(
        "define {} {}({}) {{\nentry:\n",
        sig.ret.ir(),
        ll_symbol(mangled),
        params.join(", ")
    ));
    if sig.ret == LlTy::Void {
        out.push_str(&format!("  call void {}({})\n", ll_symbol(base), call_args));
        out.push_str("  ret void\n");
    } else {
        out.push_str(&format!(
            "  %0 = call {} {}({})\n",
            sig.ret.ir(),
            ll_symbol(base),
            call_args
        ));
        out.push_str(&format!("  ret {} %0\n", sig.ret.ir()));
    }
    out.push_str("}\n\n");
    emitted_functions.push(mangled.to_string());
    true
}

fn emit_base_alias_to_mangled(
    mangled: &str,
    sig: &FnSig,
    sigs: &BTreeMap<String, FnSig>,
    out: &mut String,
    emitted_functions: &mut Vec<String>,
) -> bool {
    let Some(sep) = find_mangled_signature_separator(mangled) else {
        return false;
    };
    let base = &mangled[..sep];
    if base.is_empty() || base == mangled {
        return false;
    }
    if emitted_functions.iter().any(|n| n == base) || llvm_output_has_function(out, base) {
        return false;
    }
    let variants = sigs
        .keys()
        .filter(|n| *n == base || n.starts_with(&format!("{}__", base)))
        .collect::<Vec<_>>();
    if variants.len() != 1 || variants[0].as_str() != mangled {
        return false;
    }
    let params = sig
        .params
        .iter()
        .enumerate()
        .map(|(i, ty)| format!("{} %p{}", ty.ir(), i))
        .collect::<Vec<_>>();
    let call_args = sig
        .params
        .iter()
        .enumerate()
        .map(|(i, ty)| format!("{} %p{}", ty.ir(), i))
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!(
        "define {} {}({}) {{\nentry:\n",
        sig.ret.ir(),
        ll_symbol(base),
        params.join(", ")
    ));
    if sig.ret == LlTy::Void {
        out.push_str(&format!(
            "  call void {}({})\n",
            ll_symbol(mangled),
            call_args
        ));
        out.push_str("  ret void\n");
    } else {
        out.push_str(&format!(
            "  %0 = call {} {}({})\n",
            sig.ret.ir(),
            ll_symbol(mangled),
            call_args
        ));
        out.push_str(&format!("  ret {} %0\n", sig.ret.ir()));
    }
    out.push_str("}\n\n");
    emitted_functions.push(String::from(base));
    true
}

fn emit_fallback_linear_memory_runtime(out: &mut String) {
    out.push_str("@__nepl_fallback_mem = internal global [67108864 x i8] zeroinitializer, align 16\n");
    out.push_str("@__nepl_fallback_heap = internal global i32 16, align 4\n");
    out.push_str("define internal i32 @__nepl_fallback_alloc(i32 %size) {\n");
    out.push_str("entry:\n");
    out.push_str("  %ok = icmp sgt i32 %size, 0\n");
    out.push_str("  br i1 %ok, label %alloc, label %ret_zero\n");
    out.push_str("alloc:\n");
    out.push_str("  %heap0 = load i32, i32* @__nepl_fallback_heap, align 4\n");
    out.push_str("  %add = add i32 %size, 7\n");
    out.push_str("  %q = sdiv i32 %add, 8\n");
    out.push_str("  %aligned = mul i32 %q, 8\n");
    out.push_str("  %next = add i32 %heap0, %aligned\n");
    out.push_str("  store i32 %next, i32* @__nepl_fallback_heap, align 4\n");
    out.push_str("  ret i32 %heap0\n");
    out.push_str("ret_zero:\n");
    out.push_str("  ret i32 0\n");
    out.push_str("}\n\n");
}

fn find_mangled_signature_separator(name: &str) -> Option<usize> {
    let bytes = name.as_bytes();
    if bytes.len() < 3 {
        return None;
    }
    for i in 1..(bytes.len() - 1) {
        if bytes[i] == b'_' && bytes[i + 1] == b'_' {
            return Some(i);
        }
    }
    None
}

fn llvm_output_mentions_symbol(out: &str, sym: &str) -> bool {
    let plain = format!("@{}(", sym);
    let quoted = format!("@\"{}\"(", sym);
    out.contains(plain.as_str()) || out.contains(quoted.as_str())
}

fn build_hir_for_llvm_lowering(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> Result<(TypeCtx, HirModule), LlvmCodegenError> {
    try_build_hir_with_target(module, target, profile).map_err(|reason| LlvmCodegenError::TypecheckFailed {
        reason,
    })
}

fn try_build_hir_with_target(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> Result<(TypeCtx, HirModule), String> {
    let typed = crate::typecheck::typecheck(module, target, profile);
    let Some(typed_module) = typed.module else {
        return Err(summarize_diagnostics_for_message(&typed.diagnostics));
    };
    let mut types = typed.types;
    let hir = crate::monomorphize::monomorphize(&mut types, typed_module);
    Ok((types, hir))
}

fn collect_hir_signatures(types: &TypeCtx, module: &HirModule) -> BTreeMap<String, FnSig> {
    let mut out = BTreeMap::new();
    for f in &module.functions {
        let params = f.params.iter().map(|p| llty_for_type(types, p.ty)).collect::<Vec<_>>();
        let ret = llty_for_type(types, f.result);
        out.insert(f.name.clone(), FnSig { params, ret });
    }
    for ex in &module.externs {
        let params = ex
            .params
            .iter()
            .map(|p| llty_for_type(types, *p))
            .collect::<Vec<_>>();
        let ret = llty_for_type(types, ex.result);
        out.insert(ex.local_name.clone(), FnSig { params, ret });
    }
    out
}

fn collect_reachable_functions(module: &HirModule, entry: &str) -> Vec<String> {
    let mut function_map: BTreeMap<String, &HirFunction> = BTreeMap::new();
    for f in &module.functions {
        function_map.insert(f.name.clone(), f);
    }
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut stack = Vec::new();
    stack.push(entry.to_string());
    while let Some(name) = stack.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        let Some(func) = function_map.get(name.as_str()) else {
            continue;
        };
        let mut callees = BTreeSet::new();
        collect_callees_in_body(&func.body, &mut callees);
        for c in callees {
            if !visited.contains(c.as_str()) {
                stack.push(c);
            }
        }
    }
    visited.into_iter().collect::<Vec<_>>()
}

fn extend_reachable_with_runtime_helpers(
    reachable: &mut Vec<String>,
    module: &HirModule,
    sigs: &BTreeMap<String, FnSig>,
) {
    let mut helper_roots = Vec::new();
    let push_root = |roots: &mut Vec<String>, name: Option<&str>| {
        if let Some(n) = name {
            if !roots.iter().any(|r| r == n) {
                roots.push(String::from(n));
            }
        }
    };
    push_root(
        &mut helper_roots,
        resolve_symbol_name(sigs, "alloc", &[LlTy::I32], LlTy::I32),
    );
    push_root(
        &mut helper_roots,
        resolve_symbol_name(sigs, "dealloc", &[LlTy::I32, LlTy::I32], LlTy::Void),
    );
    push_root(
        &mut helper_roots,
        resolve_symbol_name(sigs, "realloc", &[LlTy::I32, LlTy::I32, LlTy::I32], LlTy::I32),
    );
    push_root(
        &mut helper_roots,
        resolve_symbol_name(sigs, "store_i32", &[LlTy::I32, LlTy::I32], LlTy::Void),
    );
    push_root(
        &mut helper_roots,
        resolve_symbol_name(sigs, "store_u8", &[LlTy::I32, LlTy::I32], LlTy::Void),
    );
    push_root(
        &mut helper_roots,
        resolve_symbol_name(sigs, "load_i32", &[LlTy::I32], LlTy::I32),
    );
    push_root(
        &mut helper_roots,
        resolve_symbol_name(sigs, "load_u8", &[LlTy::I32], LlTy::I32),
    );
    push_root(
        &mut helper_roots,
        resolve_symbol_name(sigs, "align8", &[LlTy::I32], LlTy::I32),
    );
    push_root(
        &mut helper_roots,
        resolve_symbol_name(sigs, "mem_size", &[], LlTy::I32),
    );
    push_root(
        &mut helper_roots,
        resolve_symbol_name(sigs, "mem_grow", &[LlTy::I32], LlTy::I32),
    );
    if helper_roots.is_empty() {
        return;
    }

    let mut function_map: BTreeMap<String, &HirFunction> = BTreeMap::new();
    for f in &module.functions {
        function_map.insert(f.name.clone(), f);
    }

    let mut seen: BTreeSet<String> = reachable.iter().cloned().collect();
    let mut stack = Vec::new();
    for root in helper_roots {
        if seen.insert(root.clone()) {
            reachable.push(root.clone());
            stack.push(root);
        }
    }
    while let Some(name) = stack.pop() {
        let Some(func) = function_map.get(name.as_str()) else {
            continue;
        };
        let mut refs = BTreeSet::new();
        collect_callees_in_body(&func.body, &mut refs);
        for callee in refs {
            if seen.insert(callee.clone()) {
                reachable.push(callee.clone());
                stack.push(callee);
            }
        }
    }
}

fn collect_callees_in_body(body: &HirBody, out: &mut BTreeSet<String>) {
    match body {
        HirBody::Block(block) => collect_callees_in_block(block, out),
        HirBody::LlvmIr(raw) => collect_callees_in_llvmir_block(raw, out),
        HirBody::Wasm(_) => {}
    }
}

fn collect_callees_in_block(block: &HirBlock, out: &mut BTreeSet<String>) {
    for line in &block.lines {
        collect_callees_in_expr(&line.expr, out);
    }
}

fn collect_callees_in_expr(expr: &HirExpr, out: &mut BTreeSet<String>) {
    match &expr.kind {
        HirExprKind::Call { callee, args } => {
            match callee {
                FuncRef::Builtin(name) | FuncRef::User(name, _) => {
                    out.insert(name.clone());
                }
                FuncRef::Trait { .. } => {}
            }
            for a in args {
                collect_callees_in_expr(a, out);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collect_callees_in_expr(cond, out);
            collect_callees_in_expr(then_branch, out);
            collect_callees_in_expr(else_branch, out);
        }
        HirExprKind::While { cond, body } => {
            collect_callees_in_expr(cond, out);
            collect_callees_in_expr(body, out);
        }
        HirExprKind::Block(b) => collect_callees_in_block(b, out),
        HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
            collect_callees_in_expr(value, out);
        }
        HirExprKind::Intrinsic { args, .. } => {
            for a in args {
                collect_callees_in_expr(a, out);
            }
        }
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => collect_callees_in_expr(inner, out),
        HirExprKind::Match { scrutinee, arms } => {
            collect_callees_in_expr(scrutinee, out);
            for arm in arms {
                collect_callees_in_expr(&arm.body, out);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(payload) = payload {
                collect_callees_in_expr(payload, out);
            }
        }
        HirExprKind::StructConstruct { fields, .. } | HirExprKind::TupleConstruct { items: fields } => {
            for f in fields {
                collect_callees_in_expr(f, out);
            }
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            collect_callees_in_expr(callee, out);
            for a in args {
                collect_callees_in_expr(a, out);
            }
        }
        HirExprKind::Var(name) => {
            if name.contains("__") || name.contains("::") {
                out.insert(name.clone());
            }
        }
        HirExprKind::FnValue(name) => {
            out.insert(name.clone());
        }
        HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit
        | HirExprKind::Drop { .. } => {}
    }
}

fn collect_callees_in_llvmir_block(
    block: &crate::ast::LlvmIrBlock,
    out: &mut BTreeSet<String>,
) {
    for line in &block.lines {
        collect_callee_from_llvmir_line(line, out);
    }
}

fn collect_callee_from_llvmir_line(line: &str, out: &mut BTreeSet<String>) {
    let trimmed = line.trim();
    if !trimmed.contains("call") {
        return;
    }
    let Some(call_pos) = trimmed.find("call") else {
        return;
    };
    let rest = &trimmed[(call_pos + 4)..];
    let Some(at_pos) = rest.find('@') else {
        return;
    };
    let after_at = &rest[(at_pos + 1)..];
    let name = if let Some(stripped) = after_at.strip_prefix('"') {
        if let Some(end_q) = stripped.find('"') {
            &stripped[..end_q]
        } else {
            return;
        }
    } else {
        let end = after_at
            .find(|c: char| c == '(' || c.is_ascii_whitespace())
            .unwrap_or(after_at.len());
        &after_at[..end]
    };
    if !name.is_empty() {
        out.insert(String::from(name));
    }
}

fn lower_hir_function(
    types: &TypeCtx,
    module: &HirModule,
    sigs: &BTreeMap<String, FnSig>,
    reachable: &BTreeSet<String>,
    memory_global: &str,
    fallback_alloc_symbol: Option<&str>,
    func: &HirFunction,
    block: &HirBlock,
) -> Result<String, LlvmCodegenError> {
    let mut exported_name = func.name.clone();
    let mut ret_ty = llty_for_type(types, func.result);
    if func.name == "main" && matches!(ret_ty, LlTy::Void) {
        exported_name = String::from("__nepl_entry_main");
        ret_ty = LlTy::Void;
    }

    let mut ctx = LowerCtx::new(
        func.name.as_str(),
        sigs,
        reachable,
        &module.string_literals,
        memory_global,
        fallback_alloc_symbol,
    );
    let mut params = Vec::new();
    for (idx, p) in func.params.iter().enumerate() {
        let pty = llty_for_type(types, p.ty);
        params.push(format!("{} %p{}", pty.ir(), idx));
    }
    ctx.push_line(&format!(
        "define {} {}({}) {{",
        ret_ty.ir(),
        ll_symbol(exported_name.as_str()),
        params.join(", ")
    ));
    ctx.push_line("entry:");

    ctx.begin_scope();
    for (idx, p) in func.params.iter().enumerate() {
        let pty = llty_for_type(types, p.ty);
        let ptr = ctx.next_tmp();
        ctx.push_line(&format!("  {} = alloca {}", ptr, pty.ir()));
        ctx.push_line(&format!(
            "  store {} %p{}, {}* {}",
            pty.ir(),
            idx,
            pty.ir(),
            ptr
        ));
        ctx.bind_local(p.name.as_str(), ptr, pty);
    }

    let ret_val = lower_hir_block(types, &mut ctx, block)?;
    match ret_ty {
        LlTy::Void => {
            ctx.push_line("  ret void");
        }
        _ => {
            if let Some(v) = ret_val {
                if v.ty == ret_ty {
                    ctx.push_line(&format!("  ret {} {}", ret_ty.ir(), v.repr));
                } else {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: func.name.clone(),
                        reason: format!("return type mismatch {:?} -> {:?}", v.ty, ret_ty),
                    });
                }
            } else {
                let zero = match ret_ty {
                    LlTy::I32 => "0",
                    LlTy::I64 => "0",
                    LlTy::F32 => "0.0",
                    LlTy::F64 => "0.0",
                    LlTy::Void => "",
                };
                ctx.push_line(&format!("  ret {} {}", ret_ty.ir(), zero));
            }
        }
    }
    ctx.end_scope();
    ctx.push_line("}");
    Ok(ctx.out)
}

fn lower_hir_block(
    types: &TypeCtx,
    ctx: &mut LowerCtx<'_>,
    block: &HirBlock,
) -> Result<Option<LlValue>, LlvmCodegenError> {
    ctx.begin_scope();
    predeclare_block_locals(types, ctx, block);
    let mut last = None;
    for line in &block.lines {
        let v = lower_hir_expr(types, ctx, &line.expr)?;
        if !line.drop_result {
            last = v;
        }
    }
    ctx.end_scope();
    Ok(last)
}

fn predeclare_block_locals(types: &TypeCtx, ctx: &mut LowerCtx<'_>, block: &HirBlock) {
    for line in &block.lines {
        if let HirExprKind::Let { name, value, .. } = &line.expr.kind {
            if ctx.lookup_local_current(name.as_str()).is_some() {
                continue;
            }
            let llty = llty_for_type(types, value.ty);
            if llty == LlTy::Void {
                continue;
            }
            let ptr = ctx.next_tmp();
            ctx.push_line(&format!("  {} = alloca {}", ptr, llty.ir()));
            ctx.bind_local(name.as_str(), ptr, llty);
        }
    }
}

fn lower_hir_expr(
    types: &TypeCtx,
    ctx: &mut LowerCtx<'_>,
    expr: &HirExpr,
) -> Result<Option<LlValue>, LlvmCodegenError> {
    match &expr.kind {
        HirExprKind::LiteralI32(v) => Ok(Some(LlValue {
            ty: LlTy::I32,
            repr: format!("{}", v),
        })),
        HirExprKind::LiteralF32(v) => Ok(Some(LlValue {
            ty: LlTy::F32,
            repr: llvm_f32_literal(*v),
        })),
        HirExprKind::LiteralBool(v) => Ok(Some(LlValue {
            ty: LlTy::I32,
            repr: if *v { String::from("1") } else { String::from("0") },
        })),
        HirExprKind::LiteralStr(id) => lower_hir_string_literal(types, ctx, *id as usize),
        HirExprKind::Unit => Ok(None),
        HirExprKind::Var(name) => {
            let Some(binding) = ctx.lookup_local_fuzzy(name.as_str()) else {
                if let Some(fid) = ctx.function_id_of(name.as_str()) {
                    return Ok(Some(LlValue {
                        ty: LlTy::I32,
                        repr: format!("{}", fid),
                    }));
                }
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: format!("unknown variable '{}'", name),
                });
            };
            let bty = binding.ty;
            let bptr = binding.ptr.clone();
            let tmp = ctx.next_tmp();
            ctx.push_line(&format!(
                "  {} = load {}, {}* {}",
                tmp,
                bty.ir(),
                bty.ir(),
                bptr
            ));
            Ok(Some(LlValue {
                ty: bty,
                repr: tmp,
            }))
        }
        HirExprKind::Let { name, value, .. } => {
            let Some(v) = lower_hir_expr(types, ctx, value)? else {
                return Ok(None);
            };
            let (ptr, pty) = if let Some(binding) = ctx.lookup_local_fuzzy(name.as_str()).cloned() {
                (binding.ptr, binding.ty)
            } else {
                let ptr = ctx.next_tmp();
                ctx.push_line(&format!("  {} = alloca {}", ptr, v.ty.ir()));
                ctx.bind_local(name.as_str(), ptr.clone(), v.ty);
                (ptr, v.ty)
            };
            if v.ty != pty {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: format!("let type mismatch {:?} -> {:?}", v.ty, pty),
                });
            }
            ctx.push_line(&format!(
                "  store {} {}, {}* {}",
                v.ty.ir(),
                v.repr,
                pty.ir(),
                ptr
            ));
            Ok(None)
        }
        HirExprKind::Set { name, value } => {
            let Some(binding) = ctx.lookup_local_fuzzy(name.as_str()).cloned() else {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: format!("set on unknown variable '{}'", name),
                });
            };
            let Some(v) = lower_hir_expr(types, ctx, value)? else {
                return Ok(None);
            };
            if v.ty != binding.ty {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: format!("set type mismatch {:?} -> {:?}", v.ty, binding.ty),
                });
            }
            ctx.push_line(&format!(
                "  store {} {}, {}* {}",
                v.ty.ir(),
                v.repr,
                binding.ty.ir(),
                binding.ptr
            ));
            Ok(None)
        }
        HirExprKind::FnValue(name) => {
            if let Some(fid) = ctx.function_id_of(name.as_str()) {
                Ok(Some(LlValue {
                    ty: LlTy::I32,
                    repr: format!("{}", fid),
                }))
            } else {
                Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: format!("unknown function value '{}'", name),
                })
            }
        }
        HirExprKind::Call { callee, args } => {
            let callee_name = match callee {
                FuncRef::Builtin(name) | FuncRef::User(name, _) => name.as_str(),
                FuncRef::Trait { trait_name, method, .. } => {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: format!("trait call {}::{} is not yet supported", trait_name, method),
                    });
                }
            };
            let mut lowered_args = Vec::new();
            for a in args {
                if let Some(v) = lower_hir_expr(types, ctx, a)? {
                    lowered_args.push(v);
                }
            }
            let sig = ctx.sigs.get(callee_name).ok_or_else(|| LlvmCodegenError::UnsupportedHirLowering {
                function: ctx.function_name.to_string(),
                reason: format!("missing function signature for '{}'", callee_name),
            })?;
            let mut args_ir = Vec::new();
            for (idx, v) in lowered_args.iter().enumerate() {
                let ty = sig.params.get(idx).copied().unwrap_or(v.ty);
                if ty != v.ty {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: format!(
                            "call argument type mismatch on '{}': expected {:?}, got {:?}",
                            callee_name, ty, v.ty
                        ),
                    });
                }
                args_ir.push(format!("{} {}", ty.ir(), v.repr));
            }
            match sig.ret {
                LlTy::Void => {
                    ctx.push_line(&format!(
                        "  call {} {}({})",
                        sig.ret.ir(),
                        ll_symbol(callee_name),
                        args_ir.join(", ")
                    ));
                    Ok(None)
                }
                ret => {
                    let tmp = ctx.next_tmp();
                    ctx.push_line(&format!(
                        "  {} = call {} {}({})",
                        tmp,
                        ret.ir(),
                        ll_symbol(callee_name),
                        args_ir.join(", ")
                    ));
                    Ok(Some(LlValue { ty: ret, repr: tmp }))
                }
            }
        }
        HirExprKind::CallIndirect {
            callee,
            params,
            result,
            args,
        } => {
            let Some(callee_v) = lower_hir_expr(types, ctx, callee)? else {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("call_indirect callee must produce a value"),
                });
            };
            if callee_v.ty != LlTy::I32 {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("call_indirect callee must be i32 function id"),
                });
            }

            let mut lowered_args = Vec::new();
            for a in args {
                let Some(v) = lower_hir_expr(types, ctx, a)? else {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("call_indirect argument must produce a value"),
                    });
                };
                lowered_args.push(v);
            }
            let param_ll = params
                .iter()
                .map(|p| llty_for_type(types, *p))
                .collect::<Vec<_>>();
            let ret_ll = llty_for_type(types, *result);
            if lowered_args.len() != param_ll.len() {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("call_indirect argument length mismatch"),
                });
            }
            for (idx, v) in lowered_args.iter().enumerate() {
                if v.ty != param_ll[idx] {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: format!(
                            "call_indirect argument type mismatch at {}: expected {:?}, got {:?}",
                            idx, param_ll[idx], v.ty
                        ),
                    });
                }
            }

            let mut candidates = Vec::new();
            for (name, sig) in ctx.sigs.iter() {
                if !ctx.reachable.contains(name) {
                    continue;
                }
                if sig.params == param_ll && sig.ret == ret_ll {
                    if let Some(fid) = ctx.function_id_of(name.as_str()) {
                        candidates.push((name.clone(), fid));
                    }
                }
            }
            if candidates.is_empty() {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("call_indirect has no matching candidate"),
                });
            }

            let end_label = ctx.next_label("calli_end");
            let default_label = ctx.next_label("calli_default");
            let mut case_labels = Vec::new();
            for _ in &candidates {
                case_labels.push(ctx.next_label("calli_case"));
            }
            let result_slot = if ret_ll != LlTy::Void {
                let s = ctx.next_tmp();
                ctx.push_line(&format!("  {} = alloca {}", s, ret_ll.ir()));
                Some(s)
            } else {
                None
            };
            ctx.push_line(&format!(
                "  switch i32 {}, label %{} [",
                callee_v.repr, default_label
            ));
            for (idx, (_, fid)) in candidates.iter().enumerate() {
                ctx.push_line(&format!("    i32 {}, label %{}", fid, case_labels[idx]));
            }
            ctx.push_line("  ]");

            let args_ir = lowered_args
                .iter()
                .map(|v| format!("{} {}", v.ty.ir(), v.repr))
                .collect::<Vec<_>>()
                .join(", ");

            for (idx, (name, _)) in candidates.iter().enumerate() {
                ctx.push_line(&format!("{}:", case_labels[idx]));
                if ret_ll == LlTy::Void {
                    ctx.push_line(&format!(
                        "  call void {}({})",
                        ll_symbol(name.as_str()),
                        args_ir
                    ));
                } else {
                    let r = ctx.next_tmp();
                    ctx.push_line(&format!(
                        "  {} = call {} {}({})",
                        r,
                        ret_ll.ir(),
                        ll_symbol(name.as_str()),
                        args_ir
                    ));
                    if let Some(slot) = result_slot.as_ref() {
                        ctx.push_line(&format!(
                            "  store {} {}, {}* {}, align 1",
                            ret_ll.ir(),
                            r,
                            ret_ll.ir(),
                            slot
                        ));
                    }
                }
                ctx.push_line(&format!("  br label %{}", end_label));
            }
            ctx.push_line(&format!("{}:", default_label));
            ctx.push_line("  unreachable");
            ctx.push_line(&format!("{}:", end_label));
            if let Some(slot) = result_slot {
                let out = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = load {}, {}* {}, align 1",
                    out,
                    ret_ll.ir(),
                    ret_ll.ir(),
                    slot
                ));
                Ok(Some(LlValue {
                    ty: ret_ll,
                    repr: out,
                }))
            } else {
                Ok(None)
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let Some(cond_v) = lower_hir_expr(types, ctx, cond)? else {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("if condition must produce a value"),
                });
            };
            if cond_v.ty != LlTy::I32 {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("if condition must be i32/bool-compatible"),
                });
            }
            let cond_i1 = ctx.next_tmp();
            ctx.push_line(&format!(
                "  {} = icmp ne i32 {}, 0",
                cond_i1, cond_v.repr
            ));
            let result_ty = llty_for_type(types, expr.ty);
            let result_slot = if result_ty != LlTy::Void {
                let slot = ctx.next_tmp();
                ctx.push_line(&format!("  {} = alloca {}", slot, result_ty.ir()));
                Some(slot)
            } else {
                None
            };
            let then_label = ctx.next_label("if_then");
            let else_label = ctx.next_label("if_else");
            let end_label = ctx.next_label("if_end");
            ctx.push_line(&format!(
                "  br i1 {}, label %{}, label %{}",
                cond_i1, then_label, else_label
            ));

            ctx.push_line(&format!("{}:", then_label));
            if let Some(tv) = lower_hir_expr(types, ctx, then_branch)? {
                if let Some(slot) = result_slot.as_ref() {
                    if tv.ty != result_ty {
                        return Err(LlvmCodegenError::UnsupportedHirLowering {
                            function: ctx.function_name.to_string(),
                            reason: String::from("then branch result type mismatch"),
                        });
                    }
                    ctx.push_line(&format!(
                        "  store {} {}, {}* {}",
                        tv.ty.ir(),
                        tv.repr,
                        tv.ty.ir(),
                        slot
                    ));
                }
            }
            ctx.push_line(&format!("  br label %{}", end_label));

            ctx.push_line(&format!("{}:", else_label));
            if let Some(ev) = lower_hir_expr(types, ctx, else_branch)? {
                if let Some(slot) = result_slot.as_ref() {
                    if ev.ty != result_ty {
                        return Err(LlvmCodegenError::UnsupportedHirLowering {
                            function: ctx.function_name.to_string(),
                            reason: String::from("else branch result type mismatch"),
                        });
                    }
                    ctx.push_line(&format!(
                        "  store {} {}, {}* {}",
                        ev.ty.ir(),
                        ev.repr,
                        ev.ty.ir(),
                        slot
                    ));
                }
            }
            ctx.push_line(&format!("  br label %{}", end_label));
            ctx.push_line(&format!("{}:", end_label));
            if let Some(slot) = result_slot {
                let tmp = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = load {}, {}* {}",
                    tmp,
                    result_ty.ir(),
                    result_ty.ir(),
                    slot
                ));
                Ok(Some(LlValue {
                    ty: result_ty,
                    repr: tmp,
                }))
            } else {
                Ok(None)
            }
        }
        HirExprKind::While { cond, body } => {
            let cond_label = ctx.next_label("while_cond");
            let body_label = ctx.next_label("while_body");
            let end_label = ctx.next_label("while_end");
            ctx.push_line(&format!("  br label %{}", cond_label));
            ctx.push_line(&format!("{}:", cond_label));
            let Some(cond_v) = lower_hir_expr(types, ctx, cond)? else {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("while condition must produce a value"),
                });
            };
            if cond_v.ty != LlTy::I32 {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("while condition must be i32/bool-compatible"),
                });
            }
            let cmp = ctx.next_tmp();
            ctx.push_line(&format!("  {} = icmp ne i32 {}, 0", cmp, cond_v.repr));
            ctx.push_line(&format!(
                "  br i1 {}, label %{}, label %{}",
                cmp, body_label, end_label
            ));
            ctx.push_line(&format!("{}:", body_label));
            let _ = lower_hir_expr(types, ctx, body)?;
            ctx.push_line(&format!("  br label %{}", cond_label));
            ctx.push_line(&format!("{}:", end_label));
            Ok(None)
        }
        HirExprKind::EnumConstruct {
            name: _,
            variant,
            payload,
            type_args: _,
        } => {
            let payload_ll = payload.as_ref().map(|p| llty_for_type(types, p.ty));
            let (payload_offset, total_size) = match payload_ll {
                Some(LlTy::I64) | Some(LlTy::F64) => (8i64, 16i32),
                Some(LlTy::Void) => (0i64, 4i32),
                Some(_) => (4i64, 8i32),
                None => (0i64, 4i32),
            };

            let alloc_name = resolve_alloc_symbol(ctx).ok_or_else(|| {
                LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("alloc function is required for enum construction"),
                }
            })?;

            let ptr = ctx.next_tmp();
            ctx.push_line(&format!(
                "  {} = call i32 {}(i32 {})",
                ptr,
                ll_symbol(alloc_name.as_str()),
                total_size
            ));

            let tag = enum_variant_tag(types, expr.ty, variant.as_str());
            let tag_ptr = ctx.linear_typed_ptr_from_i32(ptr.as_str(), LlTy::I32);
            ctx.push_line(&format!("  store i32 {}, i32* {}, align 1", tag, tag_ptr));

            if let Some(p) = payload {
                let pv = lower_hir_expr(types, ctx, p)?;
                if let Some(vty) = payload_ll {
                    if vty == LlTy::Void {
                        return Ok(Some(LlValue {
                            ty: LlTy::I32,
                            repr: ptr,
                        }));
                    }
                    let Some(pv) = pv else {
                        return Err(LlvmCodegenError::UnsupportedHirLowering {
                            function: ctx.function_name.to_string(),
                            reason: String::from("enum payload must produce a value"),
                        });
                    };
                    if pv.ty != vty {
                        return Err(LlvmCodegenError::UnsupportedHirLowering {
                            function: ctx.function_name.to_string(),
                            reason: format!(
                                "enum payload type mismatch: expected {:?}, got {:?}",
                                vty, pv.ty
                            ),
                        });
                    }
                    let base_ptr8 = ctx.linear_i8_ptr_from_i32(ptr.as_str());
                    let payload_ptr8 = ctx.next_tmp();
                    let typed_ptr = ctx.next_tmp();
                    ctx.push_line(&format!(
                        "  {} = getelementptr i8, i8* {}, i64 {}",
                        payload_ptr8, base_ptr8, payload_offset
                    ));
                    ctx.push_line(&format!(
                        "  {} = bitcast i8* {} to {}*",
                        typed_ptr,
                        payload_ptr8,
                        vty.ir()
                    ));
                    ctx.push_line(&format!(
                        "  store {} {}, {}* {}, align 1",
                        vty.ir(),
                        pv.repr,
                        vty.ir(),
                        typed_ptr
                    ));
                }
            }

            Ok(Some(LlValue {
                ty: LlTy::I32,
                repr: ptr,
            }))
        }
        HirExprKind::StructConstruct {
            name: _,
            fields,
            type_args: _,
        } => {
            let field_tys = fields
                .iter()
                .map(|f| llty_for_type(types, f.ty))
                .collect::<Vec<_>>();
            let mut offsets = Vec::with_capacity(field_tys.len());
            let mut total_size: i32 = 0;
            for ty in &field_tys {
                offsets.push(total_size as i64);
                total_size += ll_storage_size(*ty) as i32;
            }
            let alloc_name = resolve_alloc_symbol(ctx).ok_or_else(|| {
                LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("alloc function is required for struct construction"),
                }
            })?;
            let ptr = ctx.next_tmp();
            ctx.push_line(&format!(
                "  {} = call i32 {}(i32 {})",
                ptr,
                ll_symbol(alloc_name.as_str()),
                total_size
            ));
            for (idx, f) in fields.iter().enumerate() {
                let fty = field_tys[idx];
                let fv = lower_hir_expr(types, ctx, f)?;
                if fty == LlTy::Void {
                    continue;
                }
                let Some(fv) = fv else {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("struct field must produce a value"),
                    });
                };
                if fv.ty != fty {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: format!(
                            "struct field type mismatch: expected {:?}, got {:?}",
                            fty, fv.ty
                        ),
                    });
                }
                let base_ptr8 = ctx.linear_i8_ptr_from_i32(ptr.as_str());
                let field_ptr8 = ctx.next_tmp();
                let typed_ptr = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = getelementptr i8, i8* {}, i64 {}",
                    field_ptr8, base_ptr8, offsets[idx]
                ));
                ctx.push_line(&format!(
                    "  {} = bitcast i8* {} to {}*",
                    typed_ptr,
                    field_ptr8,
                    fty.ir()
                ));
                ctx.push_line(&format!(
                    "  store {} {}, {}* {}, align 1",
                    fty.ir(),
                    fv.repr,
                    fty.ir(),
                    typed_ptr
                ));
            }
            Ok(Some(LlValue {
                ty: LlTy::I32,
                repr: ptr,
            }))
        }
        HirExprKind::TupleConstruct { items } => {
            let item_tys = items
                .iter()
                .map(|v| llty_for_type(types, v.ty))
                .collect::<Vec<_>>();
            let mut offsets = Vec::with_capacity(item_tys.len());
            let mut total_size: i32 = 0;
            for ty in &item_tys {
                offsets.push(total_size as i64);
                total_size += ll_storage_size(*ty) as i32;
            }
            let alloc_name = resolve_alloc_symbol(ctx).ok_or_else(|| {
                LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("alloc function is required for tuple construction"),
                }
            })?;
            let ptr = ctx.next_tmp();
            ctx.push_line(&format!(
                "  {} = call i32 {}(i32 {})",
                ptr,
                ll_symbol(alloc_name.as_str()),
                total_size
            ));
            for (idx, item) in items.iter().enumerate() {
                let ity = item_tys[idx];
                let iv = lower_hir_expr(types, ctx, item)?;
                if ity == LlTy::Void {
                    continue;
                }
                let Some(iv) = iv else {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("tuple item must produce a value"),
                    });
                };
                if iv.ty != ity {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: format!(
                            "tuple item type mismatch: expected {:?}, got {:?}",
                            ity, iv.ty
                        ),
                    });
                }
                let base_ptr8 = ctx.linear_i8_ptr_from_i32(ptr.as_str());
                let item_ptr8 = ctx.next_tmp();
                let typed_ptr = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = getelementptr i8, i8* {}, i64 {}",
                    item_ptr8, base_ptr8, offsets[idx]
                ));
                ctx.push_line(&format!(
                    "  {} = bitcast i8* {} to {}*",
                    typed_ptr,
                    item_ptr8,
                    ity.ir()
                ));
                ctx.push_line(&format!(
                    "  store {} {}, {}* {}, align 1",
                    ity.ir(),
                    iv.repr,
                    ity.ir(),
                    typed_ptr
                ));
            }
            Ok(Some(LlValue {
                ty: LlTy::I32,
                repr: ptr,
            }))
        }
        HirExprKind::Match { scrutinee, arms } => {
            let Some(scr_v) = lower_hir_expr(types, ctx, scrutinee)? else {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("match scrutinee must produce a value"),
                });
            };
            if scr_v.ty != LlTy::I32 {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("match scrutinee must be enum pointer (i32)"),
                });
            }
            if arms.is_empty() {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("match must have at least one arm"),
                });
            }

            let scr_ptr = ctx.linear_typed_ptr_from_i32(scr_v.repr.as_str(), LlTy::I32);
            let tag = ctx.next_tmp();
            ctx.push_line(&format!("  {} = load i32, i32* {}, align 1", tag, scr_ptr));

            let result_ty = llty_for_type(types, expr.ty);
            let result_slot = if result_ty != LlTy::Void {
                let slot = ctx.next_tmp();
                ctx.push_line(&format!("  {} = alloca {}", slot, result_ty.ir()));
                Some(slot)
            } else {
                None
            };
            let end_label = ctx.next_label("match_end");
            let default_label = ctx.next_label("match_default");
            let mut arm_labels = Vec::with_capacity(arms.len());
            for _ in arms {
                arm_labels.push(ctx.next_label("match_arm"));
            }

            ctx.push_line(&format!("  switch i32 {}, label %{} [", tag, default_label));
            for (idx, arm) in arms.iter().enumerate() {
                let arm_tag = enum_variant_tag(types, scrutinee.ty, arm.variant.as_str());
                ctx.push_line(&format!("    i32 {}, label %{}", arm_tag, arm_labels[idx]));
            }
            ctx.push_line("  ]");

            for (idx, arm) in arms.iter().enumerate() {
                ctx.push_line(&format!("{}:", arm_labels[idx]));
                ctx.begin_scope();
                if let Some(bind) = &arm.bind_local {
                    if let Some(payload_ty) =
                        enum_variant_payload(types, scrutinee.ty, arm.variant.as_str())
                    {
                        let payload_ll = llty_for_type(types, payload_ty);
                        if payload_ll == LlTy::Void {
                            // unit payload は実体を持たないため束縛しない
                            // （_ 以外への束縛利用は後段で拡張する）
                            // 現時点では match 評価の継続のみ行う。
                        } else {
                        let payload_offset = match payload_ll {
                            LlTy::I64 | LlTy::F64 => 8,
                            _ => 4,
                        };
                        let base_ptr8 = ctx.linear_i8_ptr_from_i32(scr_v.repr.as_str());
                        let payload_ptr8 = ctx.next_tmp();
                        ctx.push_line(&format!(
                            "  {} = getelementptr i8, i8* {}, i64 {}",
                            payload_ptr8, base_ptr8, payload_offset
                        ));

                        let local_ptr = ctx.next_tmp();
                        let local_val = if matches!(types.get(types.resolve_id(payload_ty)), TypeKind::U8)
                        {
                            let p = ctx.next_tmp();
                            let raw = ctx.next_tmp();
                            let z = ctx.next_tmp();
                            ctx.push_line(&format!("  {} = bitcast i8* {} to i8*", p, payload_ptr8));
                            ctx.push_line(&format!("  {} = load i8, i8* {}, align 1", raw, p));
                            ctx.push_line(&format!("  {} = zext i8 {} to i32", z, raw));
                            z
                        } else {
                            let typed_ptr = ctx.next_tmp();
                            let loaded = ctx.next_tmp();
                            ctx.push_line(&format!(
                                "  {} = bitcast i8* {} to {}*",
                                typed_ptr,
                                payload_ptr8,
                                payload_ll.ir()
                            ));
                            ctx.push_line(&format!(
                                "  {} = load {}, {}* {}, align 1",
                                loaded,
                                payload_ll.ir(),
                                payload_ll.ir(),
                                typed_ptr
                            ));
                            loaded
                        };
                        ctx.push_line(&format!("  {} = alloca {}", local_ptr, payload_ll.ir()));
                        ctx.push_line(&format!(
                            "  store {} {}, {}* {}, align 1",
                            payload_ll.ir(),
                            local_val,
                            payload_ll.ir(),
                            local_ptr
                        ));
                        ctx.bind_local(bind.as_str(), local_ptr, payload_ll);
                        }
                    }
                }

                let arm_val = lower_hir_expr(types, ctx, &arm.body)?;
                if let Some(slot) = result_slot.as_ref() {
                    let Some(v) = arm_val else {
                        ctx.end_scope();
                        continue;
                    };
                    if v.ty != result_ty {
                        return Err(LlvmCodegenError::UnsupportedHirLowering {
                            function: ctx.function_name.to_string(),
                            reason: format!(
                                "match arm result type mismatch: expected {:?}, got {:?}",
                                result_ty, v.ty
                            ),
                        });
                    }
                    ctx.push_line(&format!(
                        "  store {} {}, {}* {}, align 1",
                        v.ty.ir(),
                        v.repr,
                        v.ty.ir(),
                        slot
                    ));
                }
                ctx.end_scope();
                ctx.push_line(&format!("  br label %{}", end_label));
            }

            ctx.push_line(&format!("{}:", default_label));
            ctx.push_line("  unreachable");

            ctx.push_line(&format!("{}:", end_label));
            if let Some(slot) = result_slot {
                let tmp = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = load {}, {}* {}, align 1",
                    tmp,
                    result_ty.ir(),
                    result_ty.ir(),
                    slot
                ));
                Ok(Some(LlValue {
                    ty: result_ty,
                    repr: tmp,
                }))
            } else {
                Ok(None)
            }
        }
        HirExprKind::Block(block) => lower_hir_block(types, ctx, block),
        HirExprKind::Intrinsic {
            name,
            type_args,
            args,
        } => {
            if name == "size_of" || name == "align_of" {
                if let Some(ty) = type_args.first() {
                    let size = match types.get(types.resolve_id(*ty)) {
                        TypeKind::U8 => 1,
                        TypeKind::Named(ref n) if n == "i64" || n == "f64" => 8,
                        TypeKind::Unit => 0,
                        _ => 4,
                    };
                    return Ok(Some(LlValue {
                        ty: LlTy::I32,
                        repr: format!("{}", size),
                    }));
                }
            }
            if name == "load" {
                if type_args.len() != 1 || args.len() != 1 {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic load requires one type arg and one value arg"),
                    });
                }
                let Some(ptr_v) = lower_hir_expr(types, ctx, &args[0])? else {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic load pointer must produce a value"),
                    });
                };
                if ptr_v.ty != LlTy::I32 {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic load pointer must be i32"),
                    });
                }
                let ty_id = types.resolve_id(type_args[0]);
                let ty_kind = types.get(ty_id);
                if matches!(ty_kind, TypeKind::U8) {
                    let p_ptr = ctx.linear_i8_ptr_from_i32(ptr_v.repr.as_str());
                    let raw = ctx.next_tmp();
                    let out = ctx.next_tmp();
                    ctx.push_line(&format!("  {} = load i8, i8* {}, align 1", raw, p_ptr));
                    ctx.push_line(&format!("  {} = zext i8 {} to i32", out, raw));
                    return Ok(Some(LlValue {
                        ty: LlTy::I32,
                        repr: out,
                    }));
                }
                let out_ty = llty_for_type(types, ty_id);
                let p_ptr = ctx.linear_typed_ptr_from_i32(ptr_v.repr.as_str(), out_ty);
                let out = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = load {}, {}* {}, align 1",
                    out,
                    out_ty.ir(),
                    out_ty.ir(),
                    p_ptr
                ));
                return Ok(Some(LlValue {
                    ty: out_ty,
                    repr: out,
                }));
            }
            if name == "store" {
                if type_args.len() != 1 || args.len() != 2 {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic store requires one type arg and two value args"),
                    });
                }
                let Some(ptr_v) = lower_hir_expr(types, ctx, &args[0])? else {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic store pointer must produce a value"),
                    });
                };
                let Some(val_v) = lower_hir_expr(types, ctx, &args[1])? else {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic store value must produce a value"),
                    });
                };
                if ptr_v.ty != LlTy::I32 {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic store pointer must be i32"),
                    });
                }
                let ty_id = types.resolve_id(type_args[0]);
                let ty_kind = types.get(ty_id);
                if matches!(ty_kind, TypeKind::U8) {
                    if val_v.ty != LlTy::I32 {
                        return Err(LlvmCodegenError::UnsupportedHirLowering {
                            function: ctx.function_name.to_string(),
                            reason: String::from("intrinsic store<u8> expects i32 value"),
                        });
                    }
                    let p_ptr = ctx.linear_i8_ptr_from_i32(ptr_v.repr.as_str());
                    let b = ctx.next_tmp();
                    ctx.push_line(&format!("  {} = trunc i32 {} to i8", b, val_v.repr));
                    ctx.push_line(&format!("  store i8 {}, i8* {}, align 1", b, p_ptr));
                    return Ok(None);
                }
                let store_ty = llty_for_type(types, ty_id);
                if val_v.ty != store_ty {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: format!(
                            "intrinsic store type mismatch: expected {:?}, got {:?}",
                            store_ty, val_v.ty
                        ),
                    });
                }
                let p_ptr = ctx.linear_typed_ptr_from_i32(ptr_v.repr.as_str(), store_ty);
                ctx.push_line(&format!(
                    "  store {} {}, {}* {}, align 1",
                    store_ty.ir(),
                    val_v.repr,
                    store_ty.ir(),
                    p_ptr
                ));
                return Ok(None);
            }
            if name == "unreachable" {
                ctx.push_line("  unreachable");
                return Ok(None);
            }
            if name == "add" {
                if args.len() != 2 {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic add expects two arguments"),
                    });
                }
                let Some(a) = lower_hir_expr(types, ctx, &args[0])? else {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic add lhs must produce a value"),
                    });
                };
                let Some(b) = lower_hir_expr(types, ctx, &args[1])? else {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic add rhs must produce a value"),
                    });
                };
                if a.ty != LlTy::I32 || b.ty != LlTy::I32 {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic add currently supports i32 only"),
                    });
                }
                let out = ctx.next_tmp();
                ctx.push_line(&format!("  {} = add i32 {}, {}", out, a.repr, b.repr));
                return Ok(Some(LlValue {
                    ty: LlTy::I32,
                    repr: out,
                }));
            }
            if name == "f32_to_i32" {
                if args.len() != 1 {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic f32_to_i32 expects one argument"),
                    });
                }
                let Some(v) = lower_hir_expr(types, ctx, &args[0])? else {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic f32_to_i32 value must produce a value"),
                    });
                };
                if v.ty != LlTy::F32 {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic f32_to_i32 expects f32"),
                    });
                }
                let out = ctx.next_tmp();
                ctx.push_line(&format!("  {} = fptosi float {} to i32", out, v.repr));
                return Ok(Some(LlValue {
                    ty: LlTy::I32,
                    repr: out,
                }));
            }
            if name == "i32_to_u8" {
                if args.len() != 1 {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic i32_to_u8 expects one argument"),
                    });
                }
                let Some(v) = lower_hir_expr(types, ctx, &args[0])? else {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic i32_to_u8 value must produce a value"),
                    });
                };
                if v.ty != LlTy::I32 {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic i32_to_u8 expects i32"),
                    });
                }
                let out = ctx.next_tmp();
                ctx.push_line(&format!("  {} = and i32 {}, 255", out, v.repr));
                return Ok(Some(LlValue {
                    ty: LlTy::I32,
                    repr: out,
                }));
            }
            if name == "u8_to_i32" {
                if args.len() != 1 {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic u8_to_i32 expects one argument"),
                    });
                }
                let Some(v) = lower_hir_expr(types, ctx, &args[0])? else {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic u8_to_i32 value must produce a value"),
                    });
                };
                if v.ty != LlTy::I32 {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: String::from("intrinsic u8_to_i32 expects i32"),
                    });
                }
                let out = ctx.next_tmp();
                ctx.push_line(&format!("  {} = and i32 {}, 255", out, v.repr));
                return Ok(Some(LlValue {
                    ty: LlTy::I32,
                    repr: out,
                }));
            }
            Err(LlvmCodegenError::UnsupportedHirLowering {
                function: ctx.function_name.to_string(),
                reason: format!("unsupported intrinsic '{}'", name),
            })
        }
        HirExprKind::Drop { .. } => Ok(None),
        other => Err(LlvmCodegenError::UnsupportedHirLowering {
            function: ctx.function_name.to_string(),
            reason: format!("unsupported expression kind {:?}", other),
        }),
    }
}

fn lower_hir_string_literal(
    _types: &TypeCtx,
    ctx: &mut LowerCtx<'_>,
    id: usize,
) -> Result<Option<LlValue>, LlvmCodegenError> {
    let Some(s) = ctx.strings.get(id) else {
        return Err(LlvmCodegenError::UnsupportedHirLowering {
            function: ctx.function_name.to_string(),
            reason: format!("string literal id {} was out of bounds", id),
        });
    };
    let bytes = s.as_bytes();
    let alloc_name = resolve_alloc_symbol(ctx).ok_or_else(|| LlvmCodegenError::UnsupportedHirLowering {
        function: ctx.function_name.to_string(),
        reason: String::from("alloc function is required to materialize string literals"),
    })?;
    let ptr_tmp = ctx.next_tmp();
    let total_len = (bytes.len() + 4) as i32;
    ctx.push_line(&format!(
        "  {} = call i32 {}(i32 {})",
        ptr_tmp,
        ll_symbol(alloc_name.as_str()),
        total_len
    ));
    let len_ptr = ctx.linear_typed_ptr_from_i32(ptr_tmp.as_str(), LlTy::I32);
    ctx.push_line(&format!("  store i32 {}, i32* {}, align 1", bytes.len(), len_ptr));
    for (idx, b) in bytes.iter().enumerate() {
        let off = ctx.next_tmp();
        ctx.push_line(&format!("  {} = add i32 {}, {}", off, ptr_tmp, idx + 4));
        let ptr8 = ctx.linear_i8_ptr_from_i32(off.as_str());
        ctx.push_line(&format!(
            "  store i8 {}, i8* {}, align 1",
            *b as i32, ptr8
        ));
    }
    Ok(Some(LlValue {
        ty: LlTy::I32,
        repr: ptr_tmp,
    }))
}

fn llty_for_type(types: &TypeCtx, ty: TypeId) -> LlTy {
    match types.get(types.resolve_id(ty)) {
        TypeKind::Unit | TypeKind::Never => LlTy::Void,
        TypeKind::I32 | TypeKind::U8 | TypeKind::Bool | TypeKind::Str => LlTy::I32,
        TypeKind::F32 => LlTy::F32,
        TypeKind::Named(name) if name == "i64" => LlTy::I64,
        TypeKind::Named(name) if name == "f64" => LlTy::F64,
        TypeKind::Reference(_, _) => LlTy::I32,
        TypeKind::Box(_) => LlTy::I32,
        TypeKind::Tuple { .. } => LlTy::I32,
        TypeKind::Struct { .. } => LlTy::I32,
        TypeKind::Enum { .. } => LlTy::I32,
        TypeKind::Apply { .. } => LlTy::I32,
        TypeKind::Function { .. } => LlTy::I32,
        TypeKind::Var(_) => LlTy::I32,
        TypeKind::Named(_) => LlTy::I32,
    }
}

fn ll_storage_size(ty: LlTy) -> i64 {
    match ty {
        LlTy::I64 | LlTy::F64 => 8,
        LlTy::Void => 0,
        LlTy::I32 | LlTy::F32 => 4,
    }
}

fn ll_symbol(name: &str) -> String {
    let escaped = name
        .replace('\\', "\\5C")
        .replace('"', "\\22");
    format!("@\"{}\"", escaped)
}

fn llvm_f32_literal(v: f32) -> String {
    if v.is_nan() {
        return String::from("0x7FC00000");
    }
    if v == f32::INFINITY {
        return String::from("0x7F800000");
    }
    if v == f32::NEG_INFINITY {
        return String::from("0xFF800000");
    }
    format!("{:.9e}", v)
}

fn resolve_alloc_symbol(ctx: &LowerCtx<'_>) -> Option<String> {
    resolve_symbol_name(ctx.sigs, "alloc", &[LlTy::I32], LlTy::I32)
        .map(String::from)
        .or_else(|| ctx.fallback_alloc_symbol.map(String::from))
}

fn resolve_symbol_name<'a>(
    sigs: &'a BTreeMap<String, FnSig>,
    preferred: &'a str,
    params: &[LlTy],
    ret: LlTy,
) -> Option<&'a str> {
    let signature_matches = |sig: &FnSig| sig.ret == ret && sig.params.as_slice() == params;

    if let Some(sig) = sigs.get(preferred) {
        if signature_matches(sig) {
            return Some(preferred);
        }
    }

    let mut candidates = sigs
        .iter()
        .filter_map(|(name, sig)| {
            if !signature_matches(sig) {
                return None;
            }
            if name == preferred || name.starts_with(&format!("{}__", preferred)) {
                Some(name.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return None;
    }
    candidates.sort_unstable();
    candidates.first().copied()
}

fn enum_variant_tag(ctx: &TypeCtx, enum_ty: TypeId, variant: &str) -> i32 {
    let name = if let Some(pos) = variant.rfind("::") {
        &variant[pos + 2..]
    } else {
        variant
    };
    let enum_ty = ctx.resolve_id(enum_ty);
    match ctx.get(enum_ty) {
        TypeKind::Enum { variants, .. } => variants
            .iter()
            .position(|v| v.name == name)
            .unwrap_or(0) as i32,
        TypeKind::Apply { base, .. } => enum_variant_tag(ctx, base, name),
        _ => 0,
    }
}

fn enum_variant_payload(ctx: &TypeCtx, enum_ty: TypeId, variant: &str) -> Option<TypeId> {
    let name = if let Some(pos) = variant.rfind("::") {
        &variant[pos + 2..]
    } else {
        variant
    };
    let enum_ty = ctx.resolve_id(enum_ty);
    match ctx.get(enum_ty) {
        TypeKind::Enum { variants, .. } => variants
            .iter()
            .find(|v| v.name == name)
            .and_then(|v| v.payload),
        TypeKind::Apply { base, args } => match ctx.get(base) {
            TypeKind::Enum {
                variants,
                type_params,
                ..
            } => {
                let payload = variants
                    .iter()
                    .find(|v| v.name == name)
                    .and_then(|v| v.payload);
                payload.map(|pty| {
                    if let Some(pos) = type_params.iter().position(|tp| *tp == pty) {
                        if let Some(arg) = args.get(pos) {
                            return *arg;
                        }
                    }
                    pty
                })
            }
            _ => None,
        },
        _ => None,
    }
}

fn summarize_diagnostics_for_message(diags: &[crate::diagnostic::Diagnostic]) -> String {
    let errs = diags
        .iter()
        .filter(|d| matches!(d.severity, crate::diagnostic::Severity::Error))
        .collect::<Vec<_>>();
    if errs.is_empty() {
        return String::from("no diagnostic details");
    }
    let mut uniq = BTreeSet::new();
    for d in errs.iter().take(8) {
        uniq.insert(format!(
            "{} (file={}, start={}, end={})",
            d.message,
            d.primary.span.file_id.0,
            d.primary.span.start,
            d.primary.span.end
        ));
    }
    let total = errs.len();
    let mut parts = uniq.into_iter().collect::<Vec<_>>();
    if total > parts.len() {
        parts.push(format!("... and {} more diagnostics", total - parts.len()));
    }
    parts.join(" / ")
}

fn collect_defined_functions_from_llvmir_block(
    block: &crate::ast::LlvmIrBlock,
    out: &mut Vec<String>,
) {
    for line in &block.lines {
        if let Some(name) = parse_defined_function_name(line) {
            if !out.iter().any(|n| n == name) {
                out.push(String::from(name));
            }
        }
    }
}

fn parse_defined_function_name(line: &str) -> Option<&str> {
    parse_signature_function_name(line, true)
}

fn parse_declared_or_defined_function_name(line: &str) -> Option<&str> {
    parse_signature_function_name(line, false)
}

fn parse_signature_function_name(line: &str, define_only: bool) -> Option<&str> {
    // define/declare のシグネチャ行から関数名を抽出する。
    // 例: define i32 @foo(i32 %x) {
    // 例: declare i32 @"foo"(i32 %x)
    let trimmed = line.trim_start();
    let is_define = trimmed.starts_with("define ");
    let is_declare = trimmed.starts_with("declare ");
    if define_only {
        if !is_define {
            return None;
        }
    } else if !is_define && !is_declare {
        return None;
    }
    let at = trimmed.find('@')?;
    let rest = &trimmed[(at + 1)..];
    let end = rest.find('(')?;
    let mut name = &rest[..end];
    if name.starts_with('"') && name.ends_with('"') && name.len() >= 2 {
        name = &name[1..name.len() - 1];
    }
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn llvm_output_has_function(out: &str, name: &str) -> bool {
    out.lines()
        .filter_map(parse_declared_or_defined_function_name)
        .any(|n| n == name)
}

fn collect_active_entry_names(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> Vec<String> {
    let mut pending_if: Option<bool> = None;
    let mut out = Vec::new();
    for stmt in &module.root.items {
        if let Stmt::Directive(d) = stmt {
            if let Some(allowed) = gate_allows(d, target, profile) {
                pending_if = Some(allowed);
                continue;
            }
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        if !allowed {
            continue;
        }
        if let Stmt::Directive(Directive::Entry { name }) = stmt {
            out.push(name.name.clone());
        }
    }
    out
}

fn gate_allows(d: &Directive, target: CompileTarget, profile: BuildProfile) -> Option<bool> {
    match d {
        Directive::IfTarget { target: gate, .. } => Some(target.allows(gate.as_str())),
        Directive::IfProfile { profile: p, .. } => Some(profile_allows(p.as_str(), profile)),
        _ => None,
    }
}

fn profile_allows(profile: &str, active: BuildProfile) -> bool {
    match profile {
        "debug" => matches!(active, BuildProfile::Debug),
        "release" => matches!(active, BuildProfile::Release),
        _ => false,
    }
}

fn append_llvmir_block(out: &mut String, block: &crate::ast::LlvmIrBlock) {
    for line in &block.lines {
        out.push_str(line);
        out.push('\n');
    }
    out.push('\n');
}

fn active_stmt_indices(block: &Block, target: CompileTarget, profile: BuildProfile) -> Vec<usize> {
    let mut pending_if: Option<bool> = None;
    let mut out = Vec::new();
    for (idx, stmt) in block.items.iter().enumerate() {
        if let Stmt::Directive(d) = stmt {
            if let Some(allowed) = gate_allows(d, target, profile) {
                pending_if = Some(allowed);
                continue;
            }
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        if allowed {
            out.push(idx);
        }
    }
    out
}

fn select_raw_body_from_parsed_block<'a>(
    block: &'a Block,
    target: CompileTarget,
    profile: BuildProfile,
) -> RawBodySelection<'a> {
    let mut selected: Option<RawBodySelection<'a>> = None;
    for idx in active_stmt_indices(block, target, profile) {
        match &block.items[idx] {
            Stmt::LlvmIr(raw) => {
                if selected.is_some() {
                    return RawBodySelection::Conflict;
                }
                selected = Some(RawBodySelection::Llvm(raw));
            }
            Stmt::Wasm(_) => {
                if selected.is_some() {
                    return RawBodySelection::Conflict;
                }
                selected = Some(RawBodySelection::Wasm);
            }
            Stmt::Directive(_) => {}
            _ => return RawBodySelection::None,
        }
    }
    selected.unwrap_or(RawBodySelection::None)
}

fn lower_parsed_fn_with_gates(
    name: &str,
    signature: &TypeExpr,
    params: &[Ident],
    body: &Block,
    target: CompileTarget,
    profile: BuildProfile,
) -> Option<String> {
    if !params.is_empty() {
        return None;
    }

    let result_ty = match signature {
        TypeExpr::Function { result, .. } => result.as_ref(),
        _ => return None,
    };
    if !matches!(result_ty, TypeExpr::I32) {
        return None;
    }

    let active = active_stmt_indices(body, target, profile);
    if active.len() != 1 {
        return None;
    }
    let ret_value = match &body.items[active[0]] {
        Stmt::Expr(expr) => lower_i32_literal_expr(expr)?,
        _ => return None,
    };

    Some(format!(
        "define i32 @{}() {{\nentry:\n  ret i32 {}\n}}",
        name, ret_value
    ))
}

fn lower_i32_literal_expr(expr: &PrefixExpr) -> Option<i32> {
    if expr.items.len() != 1 {
        return None;
    }
    match &expr.items[0] {
        PrefixItem::Literal(Literal::Int(text), _) => parse_i32_literal(text),
        _ => None,
    }
}

fn parse_i32_literal(text: &str) -> Option<i32> {
    if let Some(hex) = text.strip_prefix("0x") {
        i32::from_str_radix(hex, 16).ok()
    } else if let Some(hex) = text.strip_prefix("-0x") {
        i32::from_str_radix(hex, 16).ok().map(|v| -v)
    } else {
        text.parse::<i32>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Severity;
    use crate::lexer;
    use crate::parser;
    use crate::span::FileId;

    fn parse_module(src: &str) -> Module {
        let file_id = FileId(0);
        let lexed = lexer::lex(file_id, src);
        let parsed = parser::parse_tokens(file_id, lexed);
        let has_error = parsed
            .diagnostics
            .iter()
            .any(|d| matches!(d.severity, Severity::Error));
        assert!(!has_error, "parse diagnostics: {:?}", parsed.diagnostics);
        parsed.module.expect("module should parse")
    }

    #[test]
    fn emit_ll_collects_top_and_fn_blocks() {
        let src = r#"
#indent 4
#target llvm

#llvmir:
    ; module header
    target triple = "x86_64-pc-linux-gnu"

fn body <()->i32> ():
    #llvmir:
        define i32 @body() {
        entry:
            ret i32 7
        }
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module(&module).expect("llvm ir should be emitted");
        assert!(ll.contains("; module header"));
        assert!(ll.contains("define i32 @body()"));
        assert!(ll.contains("    ret i32 7"));
    }

    #[test]
    fn emit_ll_skips_unsupported_parsed_function_body() {
        let src = r#"
#target llvm
fn body <()->i32> ():
    add 1 2
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module(&module).expect("unsupported parsed function should be skipped");
        assert!(!ll.contains("define i32 @body()"));
    }

    #[test]
    fn emit_ll_supports_parsed_const_i32_function() {
        let src = r#"
#target llvm
fn c <()->i32> ():
    123
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module(&module).expect("const i32 function should be lowered");
        assert!(ll.contains("define i32 @c()"));
        assert!(ll.contains("ret i32 123"));
    }

    #[test]
    fn emit_ll_respects_if_target_gate() {
        let src = r#"
#target llvm
#if[target=wasm]
fn w <()->i32> ():
    #wasm:
        i32.const 1

#if[target=llvm]
fn l <()->i32> ():
    #llvmir:
        define i32 @l() {
        entry:
            ret i32 9
        }
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module_for_target(&module, CompileTarget::Llvm, BuildProfile::Debug)
            .expect("llvm-gated items should compile");
        assert!(ll.contains("define i32 @l()"));
        assert!(!ll.contains("define i32 @w()"));
    }

    #[test]
    fn emit_ll_supports_function_body_if_target_raw() {
        let src = r#"
#target llvm
fn f <()->i32> ():
    #if[target=wasm]
    #wasm:
        i32.const 1
    #if[target=llvm]
    #llvmir:
        define i32 @f() {
        entry:
            ret i32 42
        }
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module_for_target(&module, CompileTarget::Llvm, BuildProfile::Debug)
            .expect("llvm raw function body should be selected");
        assert!(ll.contains("define i32 @f()"));
        assert!(ll.contains("ret i32 42"));
    }

    #[test]
    fn emit_ll_rejects_entry_with_wasm_body() {
        let src = r#"
#target llvm
#entry main
fn main <()->i32> ():
    #wasm:
        i32.const 1
"#;
        let module = parse_module(src);
        let err = emit_ll_from_module(&module).expect_err("entry with #wasm body must fail");
        assert_eq!(
            err,
            LlvmCodegenError::UnsupportedWasmBody {
                function: "main".to_string()
            }
        );
    }

    #[test]
    fn emit_ll_generates_main_bridge_from_entry() {
        let src = r#"
#target llvm
#entry boot
fn boot <()->i32> ():
    #llvmir:
        define i32 @boot() {
        entry:
            ret i32 9
        }
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module(&module).expect("entry bridge should be emitted");
        assert!(ll.contains("define i32 @boot()"));
        assert!(ll.contains("define i32 @main()"));
        assert!(ll.contains("call i32 @boot()"));
    }

}
