//! # Writer
//! 

use crate::asm::scope::Scope;
use std::fs::File;
use std::io::Write;

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
            self.op2("sw", "ra", &format!("{}(sp)", offset));
        }
    }

    pub fn epilogue(&mut self, scope: &Scope) {
        if scope.caller() {
            let offset = (scope.total_slots() - 1) * 4;
            self.op2("lw", "ra", &format!("{}(sp)", offset));
        }
        
        let slots = scope.total_slots();
        if slots < 512 {
            self.op3("addi", "sp", "sp", &format!("{}", slots * 4));
        }
        else {
            self.op2("li", "t0", &format!("{}", slots * 4));
            self.op3("add", "sp", "sp", "t0");
        }
    }
}
