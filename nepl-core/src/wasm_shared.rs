extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::hir::{FuncRef, HirBody, HirExpr, HirExprKind, HirFunction, HirModule, HirParam};
use crate::types::{TypeCtx, TypeId, TypeKind};
use wasm_encoder::ValType;

fn valtype(kind: &TypeKind) -> Option<ValType> {
    match kind {
        TypeKind::Unit => None,
        TypeKind::I32 | TypeKind::U8 | TypeKind::Bool | TypeKind::Str => Some(ValType::I32),
        TypeKind::F32 => Some(ValType::F32),
        TypeKind::Enum { .. } | TypeKind::Struct { .. } | TypeKind::Tuple { .. } => {
            Some(ValType::I32)
        }
        TypeKind::Reference(_, _) | TypeKind::Box(_) => Some(ValType::I32),
        TypeKind::Function { .. } => Some(ValType::I32),
        TypeKind::Var(_) => Some(ValType::I32),
        TypeKind::Named(name) => match name.as_str() {
            "i64" => Some(ValType::I64),
            "f64" => Some(ValType::F64),
            _ => Some(ValType::I32),
        },
        TypeKind::Apply { .. } => Some(ValType::I32),
        _ => None,
    }
}

pub(crate) fn wasm_sig(
    ctx: &TypeCtx,
    result: TypeId,
    params: &[HirParam],
) -> Option<(Vec<ValType>, Vec<ValType>)> {
    let mut param_types = Vec::new();
    for p in params {
        let vk = ctx.get(ctx.resolve_id(p.ty));
        if let Some(v) = valtype(&vk) {
            param_types.push(v);
        } else {
            return None;
        }
    }
    let res_kind = ctx.get(ctx.resolve_id(result));
    let res = if let Some(v) = valtype(&res_kind) {
        vec![v]
    } else {
        if !matches!(res_kind, TypeKind::Unit) {
            return None;
        }
        Vec::new()
    };
    Some((param_types, res))
}

pub(crate) fn wasm_sig_ids(
    ctx: &TypeCtx,
    result: TypeId,
    params: &[TypeId],
) -> Option<(Vec<ValType>, Vec<ValType>)> {
    let mut param_types = Vec::new();
    for p in params {
        let vk = ctx.get(ctx.resolve_id(*p));
        if let Some(v) = valtype(&vk) {
            param_types.push(v);
        } else {
            return None;
        }
    }
    let res_kind = ctx.get(ctx.resolve_id(result));
    let res = if let Some(v) = valtype(&res_kind) {
        vec![v]
    } else {
        if !matches!(res_kind, TypeKind::Unit) {
            return None;
        }
        Vec::new()
    };
    Some((param_types, res))
}

fn has_unbound_type_var(ctx: &TypeCtx, ty: TypeId) -> bool {
    let resolved = ctx.resolve_id(ty);
    match ctx.get(resolved) {
        TypeKind::Var(tv) => match tv.binding {
            Some(next) => has_unbound_type_var(ctx, next),
            None => true,
        },
        TypeKind::Enum { type_params, .. } => type_params.iter().any(|t| has_unbound_type_var(ctx, *t)),
        TypeKind::Struct {
            type_params, fields, ..
        } => {
            type_params.iter().any(|t| has_unbound_type_var(ctx, *t))
                || fields.iter().any(|t| has_unbound_type_var(ctx, *t))
        }
        TypeKind::Tuple { items } => items.iter().any(|t| has_unbound_type_var(ctx, *t)),
        TypeKind::Function {
            type_params,
            params,
            result,
            ..
        } => {
            type_params.iter().any(|t| has_unbound_type_var(ctx, *t))
                || params.iter().any(|t| has_unbound_type_var(ctx, *t))
                || has_unbound_type_var(ctx, result)
        }
        TypeKind::Apply { base, args } => {
            has_unbound_type_var(ctx, base) || args.iter().any(|t| has_unbound_type_var(ctx, *t))
        }
        TypeKind::Box(inner) | TypeKind::Reference(inner, _) => has_unbound_type_var(ctx, inner),
        _ => false,
    }
}

pub(crate) fn should_skip_wasm_codegen_for_generic(ctx: &TypeCtx, f: &HirFunction) -> bool {
    let fty = ctx.get(ctx.resolve_id(f.func_ty));
    if let TypeKind::Function {
        type_params,
        params,
        result,
        ..
    } = fty
    {
        if !type_params.is_empty() {
            return true;
        }
        if has_unbound_type_var(ctx, result) {
            return true;
        }
        for p in params {
            if has_unbound_type_var(ctx, p) {
                return true;
            }
        }
    }
    false
}

fn collect_called_functions_from_expr(
    expr: &HirExpr,
    out: &mut BTreeSet<String>,
    has_indirect: &mut bool,
) {
    match &expr.kind {
        HirExprKind::Call { callee, args } => {
            if let FuncRef::User(name, _) = callee {
                out.insert(name.clone());
            }
            for a in args {
                collect_called_functions_from_expr(a, out, has_indirect);
            }
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            *has_indirect = true;
            collect_called_functions_from_expr(callee, out, has_indirect);
            for a in args {
                collect_called_functions_from_expr(a, out, has_indirect);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collect_called_functions_from_expr(cond, out, has_indirect);
            collect_called_functions_from_expr(then_branch, out, has_indirect);
            collect_called_functions_from_expr(else_branch, out, has_indirect);
        }
        HirExprKind::While { cond, body } => {
            collect_called_functions_from_expr(cond, out, has_indirect);
            collect_called_functions_from_expr(body, out, has_indirect);
        }
        HirExprKind::Match { scrutinee, arms } => {
            collect_called_functions_from_expr(scrutinee, out, has_indirect);
            for arm in arms {
                collect_called_functions_from_expr(&arm.body, out, has_indirect);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(p) = payload {
                collect_called_functions_from_expr(p, out, has_indirect);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for f in fields {
                collect_called_functions_from_expr(f, out, has_indirect);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for i in items {
                collect_called_functions_from_expr(i, out, has_indirect);
            }
        }
        HirExprKind::Block(b) => {
            for line in &b.lines {
                collect_called_functions_from_expr(&line.expr, out, has_indirect);
            }
        }
        HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
            collect_called_functions_from_expr(value, out, has_indirect);
        }
        HirExprKind::Intrinsic { args, .. } => {
            for a in args {
                collect_called_functions_from_expr(a, out, has_indirect);
            }
        }
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
            collect_called_functions_from_expr(inner, out, has_indirect);
        }
        HirExprKind::Var(name) | HirExprKind::FnValue(name) => {
            out.insert(name.clone());
        }
        HirExprKind::Unit
        | HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Drop { .. } => {}
    }
}

pub(crate) fn collect_reachable_wasm_functions(module: &HirModule) -> BTreeSet<String> {
    let all_names: BTreeSet<String> = module.functions.iter().map(|f| f.name.clone()).collect();
    if all_names.is_empty() {
        return all_names;
    }

    let mut roots = BTreeSet::new();
    if let Some(entry) = &module.entry {
        if all_names.contains(entry) {
            roots.insert(entry.clone());
        }
    }
    if roots.is_empty() {
        return all_names;
    }

    let mut map: BTreeMap<String, &HirFunction> = BTreeMap::new();
    for f in &module.functions {
        map.insert(f.name.clone(), f);
    }

    let mut reachable = BTreeSet::new();
    let mut stack: Vec<String> = roots.iter().cloned().collect();
    let mut has_indirect = false;
    let resolve_name = |name: &str| -> String {
        if all_names.contains(name) {
            return name.to_string();
        }
        let mut prefix = String::from(name);
        prefix.push_str("__");
        let mut found: Option<String> = None;
        for cand in &all_names {
            if cand.starts_with(&prefix) {
                if found.is_some() {
                    return name.to_string();
                }
                found = Some(cand.clone());
            }
        }
        found.unwrap_or_else(|| name.to_string())
    };

    while let Some(name) = stack.pop() {
        let resolved_name = resolve_name(&name);
        if !reachable.insert(resolved_name.clone()) {
            continue;
        }
        let Some(func) = map.get(&resolved_name) else {
            continue;
        };
        if let HirBody::Block(b) = &func.body {
            let mut called = BTreeSet::new();
            for line in &b.lines {
                collect_called_functions_from_expr(&line.expr, &mut called, &mut has_indirect);
            }
            for callee in called {
                let resolved_callee = resolve_name(&callee);
                if all_names.contains(&resolved_callee) && !reachable.contains(&resolved_callee) {
                    stack.push(resolved_callee);
                }
            }
        }
    }

    if has_indirect {
        return all_names;
    }
    reachable
}

fn collect_indirect_sigs(expr: &HirExpr, out: &mut Vec<(Vec<ValType>, Vec<ValType>)>, ctx: &TypeCtx) {
    match &expr.kind {
        HirExprKind::CallIndirect {
            callee,
            params,
            result,
            args,
        } => {
            let mut p = Vec::new();
            let mut ok = true;
            for ty in params {
                let kind = ctx.get(ctx.resolve_id(*ty));
                if let Some(vt) = valtype(&kind) {
                    p.push(vt);
                } else {
                    ok = false;
                    break;
                }
            }
            if ok {
                let res_kind = ctx.get(ctx.resolve_id(*result));
                let r = if let Some(vt) = valtype(&res_kind) {
                    vec![vt]
                } else if matches!(res_kind, TypeKind::Unit) {
                    Vec::new()
                } else {
                    Vec::new()
                };
                out.push((p, r));
            }
            collect_indirect_sigs(callee, out, ctx);
            for a in args {
                collect_indirect_sigs(a, out, ctx);
            }
        }
        HirExprKind::Call { args, .. } => {
            for a in args {
                collect_indirect_sigs(a, out, ctx);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collect_indirect_sigs(cond, out, ctx);
            collect_indirect_sigs(then_branch, out, ctx);
            collect_indirect_sigs(else_branch, out, ctx);
        }
        HirExprKind::While { cond, body } => {
            collect_indirect_sigs(cond, out, ctx);
            collect_indirect_sigs(body, out, ctx);
        }
        HirExprKind::Match { scrutinee, arms } => {
            collect_indirect_sigs(scrutinee, out, ctx);
            for arm in arms {
                collect_indirect_sigs(&arm.body, out, ctx);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(p) = payload {
                collect_indirect_sigs(p, out, ctx);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for f in fields {
                collect_indirect_sigs(f, out, ctx);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for item in items {
                collect_indirect_sigs(item, out, ctx);
            }
        }
        HirExprKind::Block(b) => {
            for line in &b.lines {
                collect_indirect_sigs(&line.expr, out, ctx);
            }
        }
        HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
            collect_indirect_sigs(value, out, ctx);
        }
        HirExprKind::Intrinsic { args, .. } => {
            for a in args {
                collect_indirect_sigs(a, out, ctx);
            }
        }
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
            collect_indirect_sigs(inner, out, ctx);
        }
        HirExprKind::Unit
        | HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Var(_)
        | HirExprKind::FnValue(_)
        | HirExprKind::Drop { .. } => {}
    }
}

pub(crate) fn collect_wasm_signature_set(
    ctx: &TypeCtx,
    module: &HirModule,
) -> BTreeSet<(Vec<ValType>, Vec<ValType>)> {
    let mut out = BTreeSet::new();
    let reachable_functions = collect_reachable_wasm_functions(module);
    for ext in &module.externs {
        if let Some((params, results)) = wasm_sig_ids(ctx, ext.result, &ext.params) {
            out.insert((params, results));
        }
    }
    for f in &module.functions {
        if !reachable_functions.contains(&f.name) {
            continue;
        }
        if let Some((params, results)) = wasm_sig(ctx, f.result, &f.params) {
            out.insert((params, results));
        }
    }
    let mut indirect_sigs = Vec::new();
    for f in &module.functions {
        if let HirBody::Block(b) = &f.body {
            for line in &b.lines {
                collect_indirect_sigs(&line.expr, &mut indirect_sigs, ctx);
            }
        }
    }
    for (params, results) in indirect_sigs {
        out.insert((params, results));
    }
    out
}

pub(crate) fn is_supported_wasm_intrinsic(name: &str) -> bool {
    matches!(
        name,
        "size_of"
            | "align_of"
            | "load"
            | "store"
            | "callsite_span"
            | "i32_to_f32"
            | "i32_to_u8"
            | "f32_to_i32"
            | "u8_to_i32"
            | "reinterpret_i32_f32"
            | "reinterpret_f32_i32"
            | "add"
            | "unreachable"
    )
}
