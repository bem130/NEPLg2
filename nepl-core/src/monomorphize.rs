#![no_std]
extern crate alloc;
extern crate std;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::hir::*;
use crate::types::{TypeCtx, TypeId, TypeKind};

pub fn monomorphize(ctx: &mut TypeCtx, module: HirModule) -> HirModule {
    let mut impl_map: BTreeMap<(String, String, TypeId), String> = BTreeMap::new();
    for imp in &module.impls {
        let ty = ctx.resolve_id(imp.target_ty);
        for m in &imp.methods {
            impl_map.insert(
                (imp.trait_name.clone(), m.name.clone(), ty),
                m.func.name.clone(),
            );
        }
    }
    let mut mono = Monomorphizer {
        ctx,
        funcs: BTreeMap::new(),
        specialized: BTreeMap::new(),
        worklist: Vec::new(),
        queued: BTreeSet::new(),
        impl_map,
    };

    for f in module.functions {
        mono.funcs.insert(f.name.clone(), f);
    }

    // Start with the entry point or all non-generic functions
    let mut initial = Vec::new();
    if let Some(entry) = &module.entry {
        initial.push(entry.clone());
    } else {
        for (name, f) in &mono.funcs {
            if let TypeKind::Function { type_params, .. } = mono.ctx.get(f.func_ty) {
                if crate::log::is_verbose() {
                    std::eprintln!(
                        "monomorphize: checking {}, params.len={}",
                        name,
                        type_params.len()
                    );
                }
                if type_params.is_empty() {
                    initial.push(name.clone());
                }
            }
        }
    }

    // Ensure runtime-required helpers are retained even if not explicitly referenced.
    // Enum/struct/tuple codegen depends on alloc being present.
    for base in ["alloc", "dealloc", "realloc"] {
        if let Some(name) = find_runtime_helper_name(&mono.funcs, base) {
            if !initial.iter().any(|n| n == &name) {
                initial.push(name);
            }
        }
    }

    for name in initial {
        if crate::log::is_verbose() {
            std::eprintln!("monomorphize: initial function {}", name);
        }
        mono.request_instantiation(name, Vec::new());
    }

    while let Some((orig_name, args)) = mono.worklist.pop() {
        mono.process_instantiation(orig_name, args);
    }

    let mut new_functions = Vec::new();
    for (_, f) in mono.specialized {
        new_functions.push(f);
    }

    HirModule {
        functions: new_functions,
        entry: module.entry,
        externs: module.externs,
        string_literals: module.string_literals,
        traits: module.traits,
        impls: module.impls,
    }
}

struct Monomorphizer<'a> {
    ctx: &'a mut TypeCtx,
    funcs: BTreeMap<String, HirFunction>,
    specialized: BTreeMap<String, HirFunction>,
    worklist: Vec<(String, Vec<TypeId>)>,
    queued: BTreeSet<String>,
    impl_map: BTreeMap<(String, String, TypeId), String>,
}

fn find_runtime_helper_name(
    funcs: &BTreeMap<String, HirFunction>,
    base: &str,
) -> Option<String> {
    if funcs.contains_key(base) {
        return Some(String::from(base));
    }
    let mut plain_prefix = String::from(base);
    plain_prefix.push_str("__");
    let mut namespaced_prefix = String::from("::");
    namespaced_prefix.push_str(base);
    namespaced_prefix.push_str("__");
    let mut namespaced_exact = String::from("::");
    namespaced_exact.push_str(base);
    for name in funcs.keys() {
        if name.starts_with(&plain_prefix)
            || name.contains(&namespaced_prefix)
            || name.ends_with(&namespaced_exact)
        {
            return Some(name.clone());
        }
    }
    None
}

impl<'a> Monomorphizer<'a> {
    fn request_instantiation(&mut self, name: String, args: Vec<TypeId>) -> String {
        let mut resolved_args = Vec::new();
        for arg in &args {
            resolved_args.push(self.ctx.resolve_id(*arg));
        }
        let args = resolved_args;
        let mangled = if args.is_empty() {
            name.clone()
        } else {
            let mut s = name.clone();
            s.push('_');
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    s.push('_');
                }
                s.push_str(&self.ctx.type_to_string(*arg));
            }
            s
        };

        if !self.specialized.contains_key(&mangled) {
            if self.queued.insert(mangled.clone()) {
                self.worklist.push((name, args));
            }
        }
        mangled
    }

    fn process_instantiation(&mut self, orig_name: String, args: Vec<TypeId>) {
        let mut resolved_args = Vec::new();
        for arg in &args {
            resolved_args.push(self.ctx.resolve_id(*arg));
        }
        let args = resolved_args;
        let mangled = if args.is_empty() {
            orig_name.clone()
        } else {
            let mut s = orig_name.clone();
            s.push('_');
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    s.push('_');
                }
                s.push_str(&self.ctx.type_to_string(*arg));
            }
            s
        };

        if self.specialized.contains_key(&mangled) {
            return;
        }

        let mut f = match self.funcs.get(&orig_name) {
            Some(f) => f.clone(),
            None => return,
        };

        let mut mapping = BTreeMap::new();
        if let TypeKind::Function { type_params, .. } = self.ctx.get(f.func_ty) {
            for (tp, arg) in type_params.iter().zip(args.iter()) {
                mapping.insert(self.ctx.resolve_id(*tp), self.ctx.resolve_id(*arg));
            }
        }

        let mut local_names: BTreeSet<String> = BTreeSet::new();
        for p in &f.params {
            local_names.insert(p.name.clone());
        }
        if let HirBody::Block(b) = &f.body {
            collect_local_names_in_block(b, &mut local_names);
        }

        // Substitute body
        f.name = mangled.clone();
        f.func_ty = self.ctx.substitute(f.func_ty, &mapping);
        f.result = self.ctx.substitute(f.result, &mapping);
        for p in &mut f.params {
            p.ty = self.ctx.substitute(p.ty, &mapping);
        }

        match &mut f.body {
            HirBody::Block(b) => self.substitute_block(b, &mapping, &local_names),
            HirBody::Wasm(_) => {} // Wasm blocks don't hold TypeIds usually
            HirBody::LlvmIr(_) => {} // LLVM IR blocks don't hold TypeIds usually
        }

        self.specialized.insert(mangled, f);
    }

    fn substitute_block(
        &mut self,
        b: &mut HirBlock,
        mapping: &BTreeMap<TypeId, TypeId>,
        local_names: &BTreeSet<String>,
    ) {
        b.ty = self.ctx.substitute(b.ty, mapping);
        for line in &mut b.lines {
            self.substitute_expr(&mut line.expr, mapping, local_names);
        }
    }

    fn substitute_expr(
        &mut self,
        expr: &mut HirExpr,
        mapping: &BTreeMap<TypeId, TypeId>,
        local_names: &BTreeSet<String>,
    ) {
        expr.ty = self.ctx.substitute(expr.ty, mapping);
        match &mut expr.kind {
            HirExprKind::Unit
            | HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_) => {}
            HirExprKind::Var(name) => {
                if local_names.contains(name) {
                    return;
                }
                if self.funcs.contains_key(name) {
                    *name = self.request_instantiation(name.clone(), Vec::new());
                } else {
                    let mut prefix = name.clone();
                    prefix.push_str("__");
                    let mut matched: Option<String> = None;
                    for cand in self.funcs.keys() {
                        if cand.starts_with(&prefix) {
                            if matched.is_some() {
                                matched = None;
                                break;
                            }
                            matched = Some(cand.clone());
                        }
                    }
                    if let Some(found) = matched {
                        *name = self.request_instantiation(found, Vec::new());
                    }
                }
            }
            HirExprKind::FnValue(name) => {
                if self.funcs.contains_key(name) {
                    *name = self.request_instantiation(name.clone(), Vec::new());
                } else {
                    let mut prefix = name.clone();
                    prefix.push_str("__");
                    let mut matched: Option<String> = None;
                    for cand in self.funcs.keys() {
                        if cand.starts_with(&prefix) {
                            if matched.is_some() {
                                matched = None;
                                break;
                            }
                            matched = Some(cand.clone());
                        }
                    }
                    if let Some(found) = matched {
                        *name = self.request_instantiation(found, Vec::new());
                    }
                }
            }
            HirExprKind::Call { callee, args } => {
                for arg in args {
                    self.substitute_expr(arg, mapping, local_names);
                }
                match callee {
                    FuncRef::User(name, type_args) => {
                        for arg in type_args.iter_mut() {
                            *arg = self.ctx.substitute(*arg, mapping);
                        }
                        // Request instantiation of the callee with concrete types
                        *name = self.request_instantiation(name.clone(), type_args.clone());
                        type_args.clear(); // Call site in WASM doesn't need type_args anymore
                    }
                    FuncRef::Trait {
                        trait_name,
                        method,
                        self_ty,
                    } => {
                        *self_ty = self.ctx.substitute(*self_ty, mapping);
                        let resolved = self.ctx.resolve_id(*self_ty);
                        let key = (
                            trait_name.clone(),
                            method.clone(),
                            resolved,
                        );
                        let mut selected = self.impl_map.get(&key).cloned();
                        if selected.is_none() {
                            let mut fallback: Option<String> = None;
                            for ((tr, meth, target_ty), func_name) in self.impl_map.iter() {
                                if tr != trait_name || meth != method {
                                    continue;
                                }
                                let mut tmp = self.ctx.clone();
                                if tmp.unify(resolved, *target_ty).is_ok() {
                                    if fallback.is_some() {
                                        fallback = None;
                                        break;
                                    }
                                    fallback = Some(func_name.clone());
                                }
                            }
                            selected = fallback;
                        }
                        if let Some(func_name) = selected {
                            let inst = self.request_instantiation(func_name, Vec::new());
                            *callee = FuncRef::User(inst, Vec::new());
                        }
                    }
                    FuncRef::Builtin(_) => {}
                }
            }
            HirExprKind::CallIndirect {
                callee,
                params,
                result,
                args,
            } => {
                self.substitute_expr(callee, mapping, local_names);
                for param in params.iter_mut() {
                    *param = self.ctx.substitute(*param, mapping);
                }
                *result = self.ctx.substitute(*result, mapping);
                for arg in args {
                    self.substitute_expr(arg, mapping, local_names);
                }
            }
            HirExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.substitute_expr(cond, mapping, local_names);
                self.substitute_expr(then_branch, mapping, local_names);
                self.substitute_expr(else_branch, mapping, local_names);
            }
            HirExprKind::While { cond, body } => {
                self.substitute_expr(cond, mapping, local_names);
                self.substitute_expr(body, mapping, local_names);
            }
        HirExprKind::Match { scrutinee, arms } => {
                self.substitute_expr(scrutinee, mapping, local_names);
                for arm in arms {
                    self.substitute_expr(&mut arm.body, mapping, local_names);
                }
            }
            HirExprKind::EnumConstruct {
                variant: _,
                type_args,
                payload,
                ..
            } => {
                for arg in type_args.iter_mut() {
                    *arg = self.ctx.substitute(*arg, mapping);
                }
                if let Some(p) = payload {
                    self.substitute_expr(p, mapping, local_names);
                }
            }
            HirExprKind::StructConstruct {
                type_args, fields, ..
            } => {
                for arg in type_args.iter_mut() {
                    *arg = self.ctx.substitute(*arg, mapping);
                }
                for f in fields {
                    self.substitute_expr(f, mapping, local_names);
                }
            }
            HirExprKind::TupleConstruct { items } => {
                for item in items {
                    self.substitute_expr(item, mapping, local_names);
                }
            }
            HirExprKind::Block(b) => self.substitute_block(b, mapping, local_names),
            HirExprKind::Let { value, .. } => self.substitute_expr(value, mapping, local_names),
            HirExprKind::Set { value, .. } => self.substitute_expr(value, mapping, local_names),
            HirExprKind::AddrOf(inner) => self.substitute_expr(inner, mapping, local_names),
            HirExprKind::Deref(inner) => self.substitute_expr(inner, mapping, local_names),
            HirExprKind::Drop { .. } => {}
            HirExprKind::Intrinsic {
                type_args,
                args,
                name: _,
            } => {
                for arg in type_args.iter_mut() {
                    *arg = self.ctx.substitute(*arg, mapping);
                }
                for arg in args {
                    self.substitute_expr(arg, mapping, local_names);
                }
            }
        }
    }
}

fn collect_local_names_in_block(block: &HirBlock, out: &mut BTreeSet<String>) {
    for line in &block.lines {
        collect_local_names_in_expr(&line.expr, out);
    }
}

fn collect_local_names_in_expr(expr: &HirExpr, out: &mut BTreeSet<String>) {
    match &expr.kind {
        HirExprKind::Let { name, value, .. } => {
            out.insert(name.clone());
            collect_local_names_in_expr(value, out);
        }
        HirExprKind::Set { value, .. } => {
            collect_local_names_in_expr(value, out);
        }
        HirExprKind::Call { args, .. } => {
            for arg in args {
                collect_local_names_in_expr(arg, out);
            }
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            collect_local_names_in_expr(callee, out);
            for arg in args {
                collect_local_names_in_expr(arg, out);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collect_local_names_in_expr(cond, out);
            collect_local_names_in_expr(then_branch, out);
            collect_local_names_in_expr(else_branch, out);
        }
        HirExprKind::While { cond, body } => {
            collect_local_names_in_expr(cond, out);
            collect_local_names_in_expr(body, out);
        }
        HirExprKind::Match { scrutinee, arms } => {
            collect_local_names_in_expr(scrutinee, out);
            for arm in arms {
                if let Some(bind) = &arm.bind_local {
                    out.insert(bind.clone());
                }
                collect_local_names_in_expr(&arm.body, out);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(p) = payload {
                collect_local_names_in_expr(p, out);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for field in fields {
                collect_local_names_in_expr(field, out);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for item in items {
                collect_local_names_in_expr(item, out);
            }
        }
        HirExprKind::Block(b) => {
            collect_local_names_in_block(b, out);
        }
        HirExprKind::Intrinsic { args, .. } => {
            for arg in args {
                collect_local_names_in_expr(arg, out);
            }
        }
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
            collect_local_names_in_expr(inner, out);
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
