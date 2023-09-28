//! # Ir
//! 
//! In this module, we presents a generator which translates program into IR string.
//! 
//! Note:
//! * in koopa ir, there seems to be infinate number of registers, as a special case of asm.
//! 

mod writer;
mod label;
mod scope;
mod translate;

use std::fs::File;
use crate::ir::writer::Writer;
use crate::ir::scope::Scope;
use crate::ir::translate::Translate;
use koopa::ir::Program;

pub fn generate_ir(program: &Program, path: &str) {
    let mut scope = Scope::new();
    let mut f = File::create(path).unwrap();
    program.translate(program, &mut scope, &mut Writer::new(&mut f));
}
