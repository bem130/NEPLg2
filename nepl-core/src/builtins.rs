#![no_std]
extern crate alloc;

use crate::ast::Effect;
use crate::types::{TypeCtx, TypeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinKind {}

#[derive(Debug, Clone)]
pub struct Builtin {
    pub name: &'static str,
    pub ty: TypeId,
    pub effect: Effect,
    pub kind: BuiltinKind,
}

pub fn builtins(_ctx: &mut TypeCtx) -> alloc::vec::Vec<Builtin> {
    alloc::vec![]
}
