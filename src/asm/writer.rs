//! # Writer
//! 

use crate::asm::scope::Scope;
use std::fs::File;
use std::io::Write;
use koopa::ir::{ Program, Value, ValueKind, Type, TypeKind };

pub struct Writer<'f> {
    f: &'f mut File,
}

impl<'f> Writer<'f> {
    pub fn new(f: &'f mut File) -> Self {
        Self { f }
    }

    pub fn note(&mut self, s: &str) {
        writeln!(self.f, "{}", s).unwrap();
    }

    pub fn line(&mut self) {
        writeln!(self.f, "").unwrap();
    }

    pub fn op1(&mut self, op: &str, dst: &str) {
        writeln!(self.f, "  {} {}", op, dst).unwrap();
    }

    pub fn op2(&mut self, op: &str, src: &str, dst: &str) {
        writeln!(self.f, "  {} {}, {}", op, src, dst).unwrap();
    }

    pub fn op3(&mut self, op: &str, dst: &str, src1: &str, src2: &str) {
        writeln!(self.f, "  {} {}, {}, {}", op, dst, src1, src2).unwrap();
    }

    pub fn prologue(&mut self, name: &str, scope: &Scope) {
        writeln!(self.f, "  .globl {}", name).unwrap();
        writeln!(self.f, "{}:", name).unwrap();

        let slots = scope.total_slots();
        if slots <= 512 {
            self.op3("addi", "sp", "sp", &format!("-{}", slots * 4));
        }
        else {
            self.op2("li", "t0", &format!("{}", slots * 4));
            self.op3("sub", "sp", "sp", "t0");
        }

        if scope.caller() {
            let offset = (scope.total_slots() - 1) * 4;
            if offset < 512 {
                self.op2("sw", "ra", &format!("{}(sp)", offset));
            }
            else {
                self.op2("li", "t0", &format!("{}", offset));
                self.op3("add", "t0", "sp", "t0");
                self.op2("sw", "ra", &format!("0(t0)"));
            }
        }
    }

    pub fn epilogue(&mut self, scope: &Scope) {
        if scope.caller() {
            let offset = (scope.total_slots() - 1) * 4;
            if offset < 512 {
                self.op2("lw", "ra", &format!("{}(sp)", offset));
            }
            else {
                self.op2("li", "t0", &format!("{}", offset));
                self.op3("add", "t0", "sp", "t0");
                self.op2("lw", "ra", &format!("0(t0)"));
            }
        }
        
        let slots = scope.total_slots();
        if slots < 512 {
            self.op3("addi", "sp", "sp", &format!("{}", slots * 4));
        }
        else {
            self.op2("li", "t0", &format!("{}", slots * 4));
            self.op3("add", "sp", "sp", "t0");
        }

        self.op1("ret", "");
    }

    pub fn aggregate(&mut self, program: &Program, value: Value) {
        match program.borrow_value(value).kind() {
            ValueKind::Integer(i) => self.note(&format!("  .word {}", i.value())),
            ValueKind::ZeroInit(_) => self.note("  .zero 4"),
            ValueKind::Aggregate(a) => {
                for elem in a.elems() {
                    self.aggregate(program, elem.clone());
                }
            }
            _ => panic!("element of this kind should not be in aggregate"),
        }
    }

    pub fn to_size(&mut self, ty: &Type) -> usize {
        match ty.kind() {
            TypeKind::Int32 => 1,
            TypeKind::Array(base, length) => self.to_size(base) * length,
            _ => panic!("type not supported"),
        }
    }
}
