//! # Asm
//! 
//! In this module, we define functions that translates the program into assembly code.
//! As is known to all, most of the optimization is done in this module.
//! 

mod label;
mod register;
mod writer;
mod scope;
mod asm;

use koopa::ir::Program;
use crate::mem::info::Info;
use crate::asm::scope::Scope;
use crate::asm::asm::Asm;
use crate::asm::writer::Writer;
use std::fs::File;

pub fn generate_asm(program: &Program, info: &mut Info, path: &str) {
    let mut scope = Scope::new();
    let mut f = File::create(path).unwrap();
    program.asm(program, &mut scope, &mut Writer::new(&mut f), info)
}
