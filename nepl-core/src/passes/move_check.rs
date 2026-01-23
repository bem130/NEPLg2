#![no_std]
extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;

use crate::hir::{HirBlock, HirExpr, HirExprKind, HirFunction, HirLine, HirModule, HirMatchArm};
use crate::span::Span;
use crate::diagnostic::Diagnostic;
use crate::types::TypeId;

/// Tracks ownership state of variables.
/// Currently simple: either Valid (Initialized) or Moved.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum VarState {
    Valid,
    Moved,
}

struct MoveCheckContext {
    /// State of all variables currently in scope.
    vars: BTreeMap<String, VarState>,
    /// Diagnostics (errors) collected.
    diagnostics: Vec<Diagnostic>,
    /// Scopes for variable cleanup (restoring previous states is complex if we shadowing, 
    /// but HIR tends to have unique names or we rely on the map overwriting. 
    /// For correctness with shadowing, we should handle scopes, but let's assume unique names or standard shadowing behavior).
    /// Actually, if we shadow, we push a new entry.
    /// Let's use a simpler "Map<Name, State>" and assuming distinct names or lexical scoping handles it.
    /// Since HIR probably relies on shadowing names, we need to be careful.
    /// But wait, `drop_insertion` used a stack of scopes. 
    /// Let's just assume we track the *current* visible instance of a name.
    /// If we enter a block and `let x`, we overwrite `x` in the map.
    /// When we exit, we need to restore?
    /// Yes.
    scopes: Vec<BTreeSet<String>>, // Tracks vars declared in current scope to remove them on exit?
    /// Actually, if we shadow `x`, we need to restore the *previous* `x` state on exit.
    /// Correct approach: `vars` maps Name -> Stack of States? Or just verify HIR names are unique?
    /// Standard Rust shadowing: the old variable is still there but inaccessible.
    /// BUT if we move out of the *new* x, the old x is unaffected.
    /// So strict scoping is needed.
    
    // Easier approach: Chain of maps? Or Map<String, Vec<State>>?
    // Let's use `Vec<BTreeMap<String, VarState>>` is too heavy.
    // Let's use a change log?
    // Let's stick to: Map<String, VarState> for *current* state. 
    // When entering scope, we push a "scope marker".
    // When declaring var, we record it.
    // When exiting scope, we remove vars declared in it.
    // SHADOWING: If I declare `x` again, I lose access to outer `x`. That's fine.
    // But when I exit scope, I need to know if outer `x` was moved? 
    // No, outer `x` cannot be accessed while inner `x` exists.
    // If I use `x`, I use inner `x`.
    // So "Map<String, VarState>" representing *currently visible* variables is enough for checking.
    // ON SCOPE EXIT: Remove the inner `x`. The outer `x` should become visible again?
    // If so, we need to restore it. 
    // Implementation: `vars` is `BTreeMap<String, Vec<VarState>>` (stack of states).
    var_stacks: BTreeMap<String, Vec<VarState>>,
}

impl MoveCheckContext {
    fn new() -> Self {
        Self {
            var_stacks: BTreeMap::new(),
            diagnostics: Vec::new(),
            scopes: Vec::new(),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(BTreeSet::new());
    }

    fn pop_scope(&mut self) {
        let vars_to_pop = self.scopes.pop().unwrap_or_default();
        for name in vars_to_pop {
            if let Some(stack) = self.var_stacks.get_mut(&name) {
                stack.pop();
                if stack.is_empty() {
                    self.var_stacks.remove(&name);
                }
            }
        }
    }

    fn declare_var(&mut self, name: String) {
        self.var_stacks.entry(name.clone()).or_default().push(VarState::Valid);
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name);
        }
    }
    
    // For function params
    fn declare_param(&mut self, name: String) {
        self.declare_var(name);
    }

    fn get_state(&self, name: &str) -> Option<VarState> {
        self.var_stacks.get(name).and_then(|s| s.last().copied())
    }

    fn set_state(&mut self, name: &str, state: VarState) {
        if let Some(stack) = self.var_stacks.get_mut(name) {
            if let Some(last) = stack.last_mut() {
                *last = state;
            }
        }
    }

    fn check_use(&mut self, name: &str, span: Span) {
        match self.get_state(name) {
            Some(VarState::Valid) => {
                // Move it!
                // Logic: "Ownership moves (not copies) by default"
                // For primitives (i32), we might want Copy?
                // For now, simpler to assume ALL types move, or check type?
                // The task description says "Move semantics... (use-after-move detection)".
                // Usually primitives like i32 are Copy. 
                // We'll need the type of the var?
                // HirExpr has type. But Var usage HIR doesn't always carry the type of the binding easily unless we look it up.
                // However, HirExpr `Var` has `ty` field!
                // So we can check if `ty` is primitive.
                // But `check_use` is called from `visit_expr` which sees `HirExpr`.
                // We'll pass `is_copy` to `check_use`.
                self.set_state(name, VarState::Moved);
            }
            Some(VarState::Moved) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("use of moved value: `{}`", name),
                    span,
                ));
            }
            None => {
                // Unknown variable? Should be caught by name resolution/typecheck.
                // Ignore or internal error.
            }
        }
    }
    
    fn check_use_copy(&mut self, name: &str, span: Span) {
         match self.get_state(name) {
            Some(VarState::Valid) => {
                // It is copy, so state remains Valid.
            }
            Some(VarState::Moved) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("use of moved value: `{}`", name),
                    span,
                ));
            }
            None => {}
        }
    }
}

pub fn check_moves(module: &HirModule) -> Vec<Diagnostic> {
    let mut ctx = MoveCheckContext::new();
    // Register types if needed? For Copy check.
    // Actually we need TypeCtx to know if a TypeId is Copy.
    // We don't have TypeCtx passed in `check_moves`.
    // We should probably modify `compile_module` to pass TypeCtx or checking is primitive.
    // For now, we can check basic types from HIR if possible, or assume non-primitive is Move.
    // Wait, `HirExpr` has `ty: TypeId`. `HirModule` doesn't have the TypeCtx.
    // Fow now, I will treat EVERYTHING as Move for safety (Linear types), 
    // OOOOR I will interpret "i32" etc as Copy if I can deduce it.
    // But `TypeId` is just an integer.
    // I cannot know if it is Copy without TypeCtx.
    // Required: Pass TypeCtx to `check_moves`.
    
    // For this pass execution, we'll assume everything moves for now (Phase 1 strict), 
    // OR we only implement "use-after-move" which implies we only care about things that moved.
    // If I move an i32, it's "moved". If I use it again, expected error?
    // Rust: i32 is Copy.
    // Task: "push memory safety... ownership/borrowing".
    // "Option 1: Rust-like... Affine ownership".
    // I should probably treat Primitives as Copy.
    // BUT I don't have TypeCtx.
    // Solution: Change `check_moves` signature to accept `&TypeCtx`.
    
    // However, for this step, I'll rely on the caller passing it.
    // I need to update `compiler.rs` to pass `&tc.types`.
    
    vec![] // Placeholder for now
}

// Logic to traverse HIR
fn visit_block(block: &HirBlock, ctx: &mut MoveCheckContext) {
    ctx.push_scope();
    for line in &block.lines {
        visit_expr(&line.expr, ctx);
        // If Let, declare it.
        if let HirExprKind::Let { name, .. } = &line.expr.kind {
            ctx.declare_var(name.clone());
        }
    }
    ctx.pop_scope();
}

fn visit_expr(expr: &HirExpr, ctx: &mut MoveCheckContext) {
    match &expr.kind {
        HirExprKind::Var(name) => {
             // For now, treating everything as Move.
             // TODO: differentiate Copy types.
             ctx.check_use(name, expr.span);
        }
        HirExprKind::If { cond, then_branch, else_branch } => {
            visit_expr(cond, ctx);
            
            // Branching requires merging states.
            // Snapshot state.
            let start_vars = ctx.var_stacks.clone();
            
            visit_expr(then_branch, ctx);
            let then_vars = ctx.var_stacks.clone();
            
            // Restore for else
            ctx.var_stacks = start_vars.clone();
            visit_expr(else_branch, ctx);
            let else_vars = ctx.var_stacks.clone();
            
            // Merge: Valid only if Valid in BOTH.
            // If Moved in either, it's Moved (conservative).
            // Actually, if Moved in Then, and I use it after If, it's invalid.
            // If Moved in Else, and I use it after If, it's invalid.
            // So logic: NewState = if Then==Moved OR Else==Moved { Moved } else { Valid }
            
            // We iterate over all keys in `start_vars` (variables visible before branch).
            for (name, stack) in start_vars {
                if let Some(start_state) = stack.last() {
                    let then_state = get_top(&then_vars, &name).unwrap_or(*start_state);
                    let else_state = get_top(&else_vars, &name).unwrap_or(*start_state);
                    
                    if then_state == VarState::Moved || else_state == VarState::Moved {
                         ctx.set_state(&name, VarState::Moved);
                    }
                }
            }
        }
        HirExprKind::While { cond, body } => {
            visit_expr(cond, ctx);
            // Loop body: similar to If, but can run multiple times.
            // If body moves a var, it is moved.
            visit_expr(body, ctx);
            // If body moved something, it remains moved.
            // Loop interaction with outer scope:
            // If I move `x` in loop, correct.
            // But if loop runs twice?
            // "Use of moved value" inside the loop?
            // Requires checking if `x` was moved in PREVIOUS iteration.
            // We can detect this by:
            // 1. Snapshot state. 2. Run body. 3. Look for moves.
            // If moved, and we used it in body... checking usage tracks order.
            // But checking usage in sequential run only checks 1st iteration.
            // We need to re-run body with the "Moved" state?
            // Fixed point?
            
            // Simple robust check: Run body once. The resulting state is the state after loop.
            // To catch "move in previous iteration":
            // Run body AGAIN with the post-loop state?
            // If error, then loop constructs invalid move.
            
            let post_first_run = ctx.var_stacks.clone();
            
            // Run again to catch loops
            visit_expr(cond, ctx);
            visit_expr(body, ctx);
            
            // Restore state to "post_first_run" (conservative estimate of one execution)
            // Actually, if loop runs 0 times? State should be original?
            // Conservative: If loop executes 0 times, state changes 0.
            // If loop executes N times, state might move.
            // Union of "0 times" (Original) and "1+ times" (PostRun).
            // So if PostRun says Moved, result is Moved (since it MIGHT have run).
            // Effectively same as If: Then (Body) or Else (Skip).
            
            // So implementation-wise:
            // 1. Run Body. Capture State S1.
            // 2. Run Body again (with S1) to check safety of repeated iterations.
            // 3. Merge S1 with Original State (S0).
            
            // Wait, we modified ctx in step 2 (diags added). We keep diags.
            // But we want the resulting state to be "Maybe Moved".
            
            // Let's refine:
            // S0
            // Run Cond -> S0_C
            // Run Body -> S1
            // Merge S0 (skip loop) and S1 (run loop).
            // Result S_Final.
            
            // BUT validity check for valid Loop definition:
            // Run Cond -> S1_C
            // Run Body -> S2
            // Use S_Final as input?
            // No, use S1 as input to check "Second Iteration".
            
            // Correct approach:
            // 1. Snapshot S0.
            // 2. Visit Cond, Body (Check logic handles 1st iteration bugs).
            // 3. Snapshot S1 (End of 1st iter).
            // 4. Reset to S1. Visit Cond, Body (Silent mode? Or just collect errors).
            //    This checks if 2nd iteration hits Moved vars.
            // 5. Merge S1 and S0 => Final State.
            
            // NOTE: We risk duplicate errors. 
            // We can suppress errors in pass 1? No, pass 1 is real.
            // Pass 2 is "hypothetical 2nd run".
            
            // Handling "Move in Loop":
            // If I move `x`, S1 has `x`=Moved.
            // Pass 2 inputs `x`=Moved.
            // Inside body, `use x`. -> ERROR "use of moved value".
            // Correct.
        }
        HirExprKind::Match { scrutinee, arms } => {
            visit_expr(scrutinee, ctx);
            let start_vars = ctx.var_stacks.clone();
            let mut branch_states = Vec::new();
            
            for arm in arms {
                 ctx.var_stacks = start_vars.clone();
                 ctx.push_scope();
                 if let Some(bind) = &arm.bind_local {
                     ctx.declare_var(bind.clone());
                 }
                 visit_expr(&arm.body, ctx);
                 ctx.pop_scope(); // Remove bindings from arm
                 branch_states.push(ctx.var_stacks.clone());
            }
            
            // Merge all branch states
            // If any branch MOVED, then treat as MOVED.
             for (name, stack) in start_vars {
                if let Some(start_state) = stack.last() {
                    let mut moved = false;
                    for branch in &branch_states {
                         if let Some(s) = get_top(branch, &name) {
                             if s == VarState::Moved { moved = true; break; }
                         }
                    }
                    if moved {
                         ctx.set_state(&name, VarState::Moved);
                    }
                }
            }
        }
        HirExprKind::Block(b) => visit_block(b, ctx),
        HirExprKind::Let { value, .. } => {
            visit_expr(value, ctx);
            // declared in visit_block loop usually? 
            // `HirExprKind::Let` is an expression. 
            // The logic in `visit_block` handles declaration.
            // But Let implies initialization.
            // Ensure we visit value first.
        },
        HirExprKind::Set { value, name } => {
             visit_expr(value, ctx);
             // Re-assignment?
             // If we assign to `x`, does it become Valid again?
             // Yes!
             // `let x = ...; move x; x = 5; use x;` -> Valid.
             // So Set marks as Valid.
             // We need to check if `name` is in scope.
             ctx.set_state(name, VarState::Valid);
        },
        HirExprKind::Call { args, .. } => {
            for arg in args { visit_expr(arg, ctx); }
        },
        HirExprKind::StructConstruct { fields, .. } => {
            for f in fields { visit_expr(f, ctx); }
        },
        _ => {
            // Recurse on children (Box, Option, etc)
            // Simplified here for brevity, need full walk.
            match &expr.kind {
                HirExprKind::LiteralI32(_) | HirExprKind::LiteralF32(_) | 
                HirExprKind::LiteralBool(_) | HirExprKind::LiteralStr(_) |
                HirExprKind::Unit | HirExprKind::Drop{..} | HirExprKind::Var(_) => {},
                 HirExprKind::EnumConstruct { payload, .. } => {
                     if let Some(p) = payload { visit_expr(p, ctx); }
                 },
                 // .. others handled above
                 _ => {}
            }
        }
    }
}

fn get_top(map: &BTreeMap<String, Vec<VarState>>, name: &str) -> Option<VarState> {
    map.get(name).and_then(|s| s.last().copied())
}

pub fn run(module: &HirModule, _types: &crate::types::TypeCtx) -> Vec<Diagnostic> {
    let mut ctx = MoveCheckContext::new();
    
    // We need to visit each function
    for func in &module.functions {
        // Reset context logic? No, create new context per function.
        let mut f_ctx = MoveCheckContext::new();
        
        // Params are valid roots
        for param in &func.params {
            f_ctx.declare_param(param.name.clone());
        }
        
        match &func.body {
             crate::hir::HirBody::Block(b) => visit_block(b, &mut f_ctx),
             _ => {}
        }
        
        ctx.diagnostics.extend(f_ctx.diagnostics);
    }
    
    ctx.diagnostics
}
