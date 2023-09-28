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
use koopa::ir::{ Program, ValueKind };

pub fn generate_mem(ast: &CompUnit) -> (Program, Info) {
    let mut program = Program::new();
    let mut info = Info::new();

    ast.create(&mut program, &mut Scope::new(), &mut info);

    for (_, func) in program.funcs() {
        println!("func: {}", func.name());
        for (bb, node) in func.layout().bbs() {
            println!("block: {}", func.dfg().bb(*bb).name().as_ref().unwrap());
            for (value, _) in node.insts() {
                match func.dfg().value(value.clone()).kind() {
                    ValueKind::Aggregate(_) => println!("  aggregate"),
                    ValueKind::Alloc(_) => println!("  alloc"),
                    ValueKind::Binary(_) => println!("  binary"),
                    ValueKind::Branch(_) => println!("  branch"),
                    ValueKind::Call(_) => println!("  call"),
                    ValueKind::GetElemPtr(_) => println!("  get elem ptr"),
                    ValueKind::GetPtr(_) => println!("  get ptr"),
                    ValueKind::GlobalAlloc(_) => println!("  global alloc"),
                    ValueKind::Integer(_) => println!("  integer"),
                    ValueKind::Jump(_) => println!("  jump"),
                    ValueKind::Load(_) => println!("  load"),
                    ValueKind::Return(_) => println!("  return"),
                    ValueKind::Store(_) => println!("  store"),
                    ValueKind::ZeroInit(_) => println!("  zero init"),
                    _ => println!("  unknown"),
                }
                println!("    {}, {}", info.info(value.clone()).unwrap().birth, info.info(value.clone()).unwrap().death);
            }
        }
    }

    (program, info)
}
