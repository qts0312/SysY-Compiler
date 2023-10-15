//! # Writer
//! 
//! Writer is a helper struct to write IR to file.
//! 

use std::fs::File;
use std::io::Write;
use crate::mem::info::Info;
use koopa::ir::{ Program, Value, ValueKind, Type, TypeKind };

pub struct Writer<'f> {
    f: &'f mut File,
    info: &'f Info,
}

impl<'f> Writer<'f> {
    pub fn new(f: &'f mut File, info: &'f Info) -> Self {
        Self {
            f,
            info,
        }
    }

    pub fn note(&mut self, s: &str) {
        writeln!(self.f, "{}", s).unwrap();
    }

    pub fn ty(&self, ty: &Type) -> String {
        match ty.kind() {
            TypeKind::Int32 => "i32".to_string(),
            TypeKind::Array(base, len) => format!("[{}, {}]", self.ty(base), len),
            TypeKind::Pointer(base) => format!("*{}", self.ty(base)),
            _ => panic!("can't turn unit or function into string")
        }
    }

    pub fn decl(&mut self, name: &str, func_ty: &Type) {
        match func_ty.kind() {
            TypeKind::Function(params_ty, ret_ty) => {
                write!(self.f, "decl {}(", name).unwrap();
                let mut first = true;
                for param_ty in params_ty {
                    if first {
                        first = false;
                    } else {
                        write!(self.f, ", ").unwrap();
                    }
                    write!(self.f, "{}", self.ty(param_ty)).unwrap();
                }

                match ret_ty.kind() {
                    TypeKind::Int32 => writeln!(self.f, "): i32").unwrap(),
                    TypeKind::Unit => writeln!(self.f, ")").unwrap(),
                    _ => panic!("we haven't implement function with other return type")
                }
            }
            _ => panic!("only can declare function")
        }
    }

    pub fn func_bg(&mut self, name: &str, func_ty: &Type, params_name: Vec<(String, Type)>) {
        match func_ty.kind() {
            TypeKind::Function(_, ret_ty) => {
                write!(self.f, "fun {}(", name).unwrap();
                let mut first = true;
                for (name, ty) in params_name {
                    if first {
                        first = false;
                    } else {
                        write!(self.f, ", ").unwrap();
                    }
                    write!(self.f, "{}: {}", name, self.ty(&ty)).unwrap();
                }
                match ret_ty.kind() {
                    TypeKind::Int32 => writeln!(self.f, "): i32 {{").unwrap(),
                    TypeKind::Unit => writeln!(self.f, ") {{").unwrap(),
                    _ => panic!("we haven't implement function with other return type")
                }
            }
            _ => panic!("only can declare function")
        }
    }

    pub fn func_ed(&mut self) {
        writeln!(self.f, "}}").unwrap();
    }

    pub fn bb_bg(&mut self, name: &str) {
        writeln!(self.f, "{}:", name).unwrap();
    }

    pub fn op3(&mut self, op: &str, dst: &str, lhs: &str, rhs: &str) {
        writeln!(self.f, "  {} = {} {}, {}", dst, op, lhs, rhs).unwrap();
    }

    pub fn ret(&mut self, val: &str) {
        writeln!(self.f, "  ret {}", val).unwrap();
    }

    pub fn load(&mut self, dst: &str, src: &str) {
        writeln!(self.f, "  {} = load {}", dst, src).unwrap();
    }

    pub fn store(&mut self, dst: &str, src: &str) {
        writeln!(self.f, "  store {}, {}", src, dst).unwrap();
    }

    pub fn branch(&mut self, cond: &str, then: &str, els: &str) {
        writeln!(self.f, "  br {}, {}, {}", cond, then, els).unwrap();
    }

    pub fn call(&mut self, dst: Option<&str>, name: &str, args: Vec<String>) {
        match dst {
            Some(dst) => {
                write!(self.f, "  {} = call {}(", dst, name).unwrap();
            }
            None => {
                write!(self.f, "  call {}(", name).unwrap();
            }
        }
        let mut first = true;
        for arg in args {
            if first {
                first = false;
            } else {
                write!(self.f, ", ").unwrap();
            }
            write!(self.f, "{}", arg).unwrap();
        }
        writeln!(self.f, ")").unwrap();
    }

    pub fn get_elem_ptr(&mut self, dst: &str, base: &str, idx: &str) {
        writeln!(self.f, "  {} = getelemptr {}, {}", dst, base, idx).unwrap();
    }

    pub fn get_ptr(&mut self, dst: &str, base: &str, idx: &str) {
        writeln!(self.f, "  {} = getptr {}, {}", dst, base, idx).unwrap();
    }

    pub fn alloc(&mut self, name: &str, ty: &Type) {
        writeln!(self.f, "  {} = alloc {}", name, self.ty(ty)).unwrap();
    }

    pub fn global_alloc(&mut self, name: &str, ty: &Type, values: &str) {
        writeln!(self.f, "global {} = alloc {}, {}", name, self.ty(ty), values).unwrap();
    }

    pub fn jump(&mut self, dst: &str) {
        writeln!(self.f, "  jump {}", dst).unwrap();
    }

    pub fn line(&mut self) {
        writeln!(self.f, "").unwrap();
    }

    pub fn to_init(&self, value: Value, program: &Program) -> String {
        match program.borrow_value(value).kind() {
            ValueKind::Integer(i) => i.value().to_string(),
            ValueKind::ZeroInit(_) => "zeroinit".to_string(),
            ValueKind::Aggregate(a) => {
                match self.info.array_info(value) {
                    Some(_) => "zeroinit".to_string(),
                    None => {
                        let mut result = "{".to_string();
                        let mut first = true;
                        for value in a.elems() {
                            if first {
                                first = false;
                            } else {
                                result.push_str(", ");
                            }
                            result.push_str(&self.to_init(*value, program));
                        }
                        result.push_str("}");
                        result
                    }
                }
            }
            _ => panic!("init shouldn't be this kind")
        }
    }
    
    pub fn to_type(&self, value: Value, program: &Program) -> Type {
        match program.borrow_value(value).kind() {
            ValueKind::Integer(_) => Type::get_i32(),
            ValueKind::ZeroInit(_) => Type::get_i32(),
            ValueKind::Aggregate(a) => {
                match self.info.array_info(value) {
                    Some(dim) => self.to_type_zero_array(dim.clone()),
                    None => Type::get_array(program.borrow_value(a.elems()[0]).ty().clone(), a.elems().len()),
                }
            }
            _ => panic!("init shouldn't be this kind")
        }
    }

    pub fn to_type_zero_array(&self, array_info: Vec<usize>) -> Type {
        if array_info.is_empty() {
            Type::get_i32()
        }
        else {
            Type::get_array(self.to_type_zero_array(array_info[1..].to_vec()), array_info[0])
        }
    }
}
