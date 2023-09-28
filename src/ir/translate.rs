//! # Translate
//! 
//! In this file, we define a trait for translating program into IR string.
//! 

#![allow(unused_assignments)]

use crate::ir::scope::{ Scope, Entry, value };
use crate::ir::writer::Writer;
use crate::tools::TurnInto;
use std::collections::HashSet;
use koopa::ir::{ Program, FunctionData, BasicBlock, ValueKind, BinaryOp, TypeKind };
use koopa::ir::entities::ValueData;
use koopa::ir::values::{ Return, Binary, Alloc, Load, Store, Branch, Jump, Call, GetElemPtr, GetPtr, GlobalAlloc };

pub trait Translate {
    fn translate(&self, program: &Program, scope: &mut Scope, w: &mut Writer);
}

impl Translate for Program {
    fn translate(&self, program: &Program, scope: &mut Scope, w: &mut Writer) {
        for value in self.inst_layout() {
            match self.borrow_value(value.clone()).kind() {
                ValueKind::GlobalAlloc(g) => g.translate(program, scope, w),
                _ => {}
            }
        }

        for func in self.func_layout() {
            scope.set_cur_func(Some(func.clone()));
            self.func(func.clone()).translate(program, scope, w);
        }
    }
}

impl Translate for FunctionData {
    fn translate(&self, program: &Program, scope: &mut Scope, w: &mut Writer) {
        // If this is a function decl
        if self.layout().bbs().is_empty() {
            w.decl(self.name(), self.ty());
            return;
        }

        let params_name: Vec<_> = self.params().iter().map(|p| {
            let label = scope.label_mut().local_var();
            scope.new_value(p.clone(), Entry::Label(label.clone()));
            (label, value!(program, scope, p.clone()).ty().clone())
        }).collect();

        w.func_bg(self.name(), self.ty(), params_name);

        // if a branch is removed by return, its then and else block will be illegal
        let mut end = false;
        let mut illegal_bbs: HashSet<BasicBlock> = HashSet::new();

        for (bb, node) in self.layout().bbs() {
            // If this block is illegal, go to the next
            if illegal_bbs.contains(bb) {
                continue;
            }

            let name = self.dfg().bb(bb.clone()).name().as_ref().unwrap();
            w.bb_bg(name);
            end = false;

            for (value, _) in node.insts() {
                if end {
                    // all the values after end should not be translated
                    match value!(program, scope, value.clone()).kind() {
                        ValueKind::Branch(b) => {
                            illegal_bbs.insert(b.true_bb());
                            illegal_bbs.insert(b.false_bb());
                        },
                        _ => {}
                    }
                    continue;
                }

                scope.set_cur_value(Some(value.clone()));
                self.dfg().value(value.clone()).translate(program, scope, w);
                match value!(program, scope, value.clone()).kind() {
                    ValueKind::Return(_) => end = true,
                    ValueKind::Jump(_) => end = true,
                    ValueKind::Branch(_) => end = true,
                    _ => {}
                }
            }

            if end == false {
                w.ret("");
            }
        }

        w.func_ed();
        scope.clear_register();
    }
}

impl Translate for ValueData {
    fn translate(&self, program: &Program, scope: &mut Scope, w: &mut Writer) {
        match self.kind() {
            ValueKind::Integer(_) => {},
            ValueKind::Return(r) => r.translate(program, scope, w),
            ValueKind::Binary(b) => b.translate(program, scope, w),
            ValueKind::Alloc(a) => a.translate(program, scope, w),
            ValueKind::Load(l) => l.translate(program, scope, w),
            ValueKind::Store(s) => s.translate(program, scope, w),
            ValueKind::Branch(b) => b.translate(program, scope, w),
            ValueKind::Jump(j) => j.translate(program, scope, w),
            ValueKind::Call(c) => c.translate(program, scope, w),
            ValueKind::GetElemPtr(g) => g.translate(program, scope, w),
            ValueKind::GetPtr(p) => p.translate(program, scope, w),
            ValueKind::GlobalAlloc(g) => g.translate(program, scope, w),
            _ => panic!("int my compiler, no these value kind")
        }
    }
}

impl Translate for Return {
    fn translate(&self, program: &Program, scope: &mut Scope, w: &mut Writer) {
        w.line();
        w.note("// return");
        let val = self.value().as_ref().map_or("".to_string(), |v| {
            match value!(program, scope, v.clone()).kind() {
                ValueKind::Integer(i) => i.value().to_string(),
                _ => scope.value(v).turn_into(())
            }
        });
        w.ret(&val);
    }
}

impl Translate for Binary {
    fn translate(&self, program: &Program, scope: &mut Scope, w: &mut Writer) {
        w.line();
        w.note("// binary");
        let lhs = match value!(program, scope, self.lhs()).kind() {
            ValueKind::Integer(i) => i.value().to_string(),
            _ => scope.value(&self.lhs()).turn_into(())
        };
        let rhs = match value!(program, scope, self.rhs()).kind() {
            ValueKind::Integer(i) => i.value().to_string(),
            _ => scope.value(&self.rhs()).turn_into(())
        };
        let dst = scope.new_register();
        scope.new_value(scope.cur_value().clone(), dst.clone());
        let dst = dst.turn_into(());

        w.op3(match self.op() {
            BinaryOp::Add => "add",
            BinaryOp::Sub => "sub",
            BinaryOp::Mul => "mul",
            BinaryOp::Div => "div",
            BinaryOp::Mod => "mod",
            BinaryOp::And => "and",
            BinaryOp::Or => "or",
            BinaryOp::Eq => "eq",
            BinaryOp::NotEq => "ne",
            BinaryOp::Lt => "lt",
            BinaryOp::Le => "le",
            BinaryOp::Gt => "gt",
            BinaryOp::Ge => "ge",
            _ => panic!("no other binary op")
        }, &dst, &lhs, &rhs);
    }
}

impl Translate for Alloc {
    fn translate(&self, program: &Program, scope: &mut Scope, w: &mut Writer) {
        w.line();
        w.note("// alloc");
        match value!(program, scope, scope.cur_value().clone()).ty().kind() {
            TypeKind::Pointer(ty) => {
                let label = match ty.kind() {
                    TypeKind::Array(_, _) => scope.label_mut().local_arr(),
                    _ => scope.label_mut().local_var(),
                };
                scope.new_value(scope.cur_value().clone(), Entry::Label(label.clone()));
                w.alloc(&label, ty);
            }
            _ => panic!("alloc should be a pointer type")
        }
    }
}

impl Translate for Load {
    fn translate(&self, _: &Program, scope: &mut Scope, w: &mut Writer) {
        w.line();
        w.note("// load");
        let dst = scope.new_register();
        let src = scope.value(&self.src());
        w.load(&dst.turn_into(()), &src.turn_into(()));
        scope.new_value(scope.cur_value().clone(), dst.clone());
    }
}

impl Translate for Store {
    fn translate(&self, program: &Program, scope: &mut Scope, w: &mut Writer) {
        w.line();
        w.note("// store");
        let src = match value!(program, scope, self.value()).kind() {
            ValueKind::Integer(i) => i.value().to_string(),
            _ => scope.value(&self.value()).turn_into(())
        };
        let dst = scope.value(&self.dest());
        w.store(&dst.turn_into(()), &src);
    }
}

impl Translate for Branch {
    fn translate(&self, program: &Program, scope: &mut Scope, w: &mut Writer) {
        w.line();
        w.note("// branch");
        let cond = match value!(program, scope, self.cond()).kind() {
            ValueKind::Integer(i) => i.value().to_string(),
            _ => scope.value(&self.cond()).turn_into(())
        };
        let then = program.func(scope.cur_func().clone()).dfg().bb(self.true_bb()).name().as_ref().unwrap();
        let els = program.func(scope.cur_func().clone()).dfg().bb(self.false_bb()).name().as_ref().unwrap();
        w.branch(&cond, then, els);
    }
}

impl Translate for Jump {
    fn translate(&self, program: &Program, scope: &mut Scope, w: &mut Writer) {
        w.line();
        w.note("// jump");
        let dst = program.func(scope.cur_func().clone()).dfg().bb(self.target()).name().as_ref().unwrap();
        w.jump(dst);
    }
}

impl Translate for Call {
    fn translate(&self, program: &Program, scope: &mut Scope, w: &mut Writer) {
        w.line();
        w.note("// call");
        let dst = scope.new_register();
        let dst_name = dst.turn_into(());
        scope.new_value(scope.cur_value().clone(), dst.clone());

        let ret_ty = match program.func(self.callee()).ty().kind() {
            TypeKind::Function(_, ret_ty) => ret_ty,
            _ => panic!("callee should be a function")
        };
        let dst = match ret_ty.kind() {
            TypeKind::Int32 => Some(dst_name.as_str()),
            _ => None,
        };
        let args: Vec<_> = self.args().iter().map(|arg| {
            match value!(program, scope, arg.clone()).kind() {
                ValueKind::Integer(i) => i.value().to_string(),
                _ => scope.value(arg).turn_into(())
            }
        }).collect();
        w.call(dst, program.func(self.callee()).name(), args);
    }
}

impl Translate for GetElemPtr {
    fn translate(&self, program: &Program, scope: &mut Scope, w: &mut Writer) {
        w.line();
        w.note("// get elem ptr");
        let dst = scope.new_register();
        scope.new_value(scope.cur_value().clone(), dst.clone());
        let src = scope.value(&self.src());
        let idx = match value!(program, scope, self.index()).kind() {
            ValueKind::Integer(i) => i.value().to_string(),
            _ => scope.value(&self.index()).turn_into(())
        };
        w.get_elem_ptr(&dst.turn_into(()), &src.turn_into(()), &idx);
    }
}

impl Translate for GetPtr {
    fn translate(&self, program: &Program, scope: &mut Scope, w: &mut Writer) {
        w.line();
        w.note("// get ptr");
        let dst = scope.new_register();
        scope.new_value(scope.cur_value().clone(), dst.clone());
        let src = scope.value(&self.src());
        let idx = match value!(program, scope, self.index()).kind() {
            ValueKind::Integer(i) => i.value().to_string(),
            _ => scope.value(&self.index()).turn_into(())
        };
        w.get_ptr(&dst.turn_into(()), &src.turn_into(()), &idx);
    }
}

impl Translate for GlobalAlloc {
    fn translate(&self, program: &Program, scope: &mut Scope, w: &mut Writer) {
        w.line();
        let ty = w.to_type(self.init(), program);
        let label = match ty.kind() {
            TypeKind::Array(_, _) => scope.label_mut().global_arr(),
            _ => scope.label_mut().global_var(),
        };
        scope.new_value(scope.cur_value().clone(), Entry::Label(label.clone()));
        w.global_alloc(&label, &ty, &w.to_init(self.init(), program));
    }
}
