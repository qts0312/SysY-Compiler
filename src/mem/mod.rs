//! # Mem
//! 
//! This module provides functions for translating compiler unit into program in memory.
//! 
//! In this module, three optimizations are performed:
//! * documenting birth and death "time" of each value to allocate registers better.
//! * when an expression can be evaluated, replace it with the number.
//! 

pub mod info;
pub mod scope;
mod label;
mod create;
mod eval;

use crate::ast::CompUnit;
use crate::mem::scope::Scope;
use crate::mem::info::Info;
use crate::mem::create::Create;
use koopa::ir::Program;

pub fn generate_mem(ast: &CompUnit) -> (Program, Info) {
    let mut program = Program::new();
    let mut info = Info::new();
    ast.create(&mut program, &mut Scope::new(), &mut info);
    (program, info)
}
