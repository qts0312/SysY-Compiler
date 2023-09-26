//! # Create
//! 
//! In this file, we define a trait for creating a program in memory and implement it for AST.
//! 

#![allow(unused_assignments)]

use crate::ast::*;
use crate::tools::{ TurnInto, global_const_array_init, local_const_array_init, global_array_init, local_array_init };
use crate::mem::scope::{ Scope, Entry, new_value, push_value, new_bb, push_bb };
use crate::mem::eval::Eval;
use crate::mem::info::Info;
use koopa::ir::builder_traits::*;
use koopa::ir::{ Program, FunctionData, Value, BinaryOp, Type, TypeKind };

pub trait Create<'ast> {
    type Out;
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out;
}

impl<'ast> Create<'ast> for CompUnit {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        let mut decl = |id: &'ast str, params_ty: Vec<Type>, ret_ty: Type| {
            let func = FunctionData::new_decl(format!("@{}", id), params_ty, ret_ty);
            scope.new_func(id, program.new_func(func));
        };
        decl("getint", vec![], Type::get_i32());
        decl("getch", vec![], Type::get_i32());
        decl("getarray", vec![Type::get_pointer(Type::get_i32())], Type::get_i32());
        decl("putint", vec![Type::get_i32()], Type::get_unit());
        decl("putch", vec![Type::get_i32()], Type::get_unit());
        decl("putarray", vec![Type::get_i32(), Type::get_pointer(Type::get_i32())], Type::get_unit());
        decl("starttime", vec![], Type::get_unit());
        decl("stoptime", vec![], Type::get_unit());

        for item in &self.items {
            item.create(program, scope, info);
        }
    }
}

impl<'ast> Create<'ast> for CompItem {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        match self {
            Self::Func(func) => func.create(program, scope, info),
            Self::Decl(decl) => decl.create(program, scope, info),
        }
    }
}

impl<'ast> Create<'ast> for Decl {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        match self {
            Self::Const(const_decl) => const_decl.create(program, scope, info),
            Self::Var(var_decl) => var_decl.create(program, scope, info),
        }
    }
}

impl<'ast> Create<'ast> for ConstDecl {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        for def in &self.defs {
            def.create(program, scope, info);
        }
    }
}

impl<'ast> Create<'ast> for ConstDef {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        if self.dims.is_empty() {
            // single variable
            let num = match &self.init {
                ConstInitVal::Exp(exp) => exp.create(program, scope, info),
                ConstInitVal::List(_) => panic!("can't initialize a single variable with a list"),
            };
            scope.new_value(&self.id, Entry::Const(num));
        } 
        else {
            // array
            let array_info: Vec<_> = self.dims.iter().map(|dim| dim.create(program, scope, info) as usize).collect();
            scope.set_array_info(array_info.clone());
            let nums = self.init.create(program, scope, info);

            if scope.is_global() {
                let init = global_const_array_init(program, nums, array_info);
                let global_alloc = program.new_value().global_alloc(init);
                info.new_info(global_alloc.clone());
                scope.new_value(&self.id, Entry::Value(global_alloc));
            }
            else {
                let alloc = new_value!(program, scope).alloc(array_info.turn_into(()));
                push_value!(program, scope, alloc.clone());
                info.new_info(alloc.clone());

                local_const_array_init(program, scope, info, nums, array_info, alloc.clone());
                scope.new_value(&self.id, Entry::Value(alloc));
            }
        }
    }
}

impl<'ast> Create<'ast> for ConstInitVal {
    type Out = Vec<i32>;
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        if scope.array_info().is_empty() {
            return vec![];
        }

        let mut result = vec![];

        let cur_array_info = scope.array_info().clone();
        let base = *(cur_array_info.last().unwrap());
        let mut count = 0;

        match self {
            Self::Exp(_) => panic!("can't initialize an array with a single expression"),
            Self::List(list) => {
                for elem in list.iter() {
                    match elem {
                        Self::Exp(exp) => {
                            result.push(exp.create(program, scope, info));
                            count += 1;
                        }
                        Self::List(_) => {
                            if count % base != 0 {
                                let remain = base - count % base;
                                result.append(&mut vec![0; remain]);
                                count += remain;
                            }

                            let mut begin = cur_array_info.len() - 1;
                            let mut len = base;

                            for num in cur_array_info.iter().rev().skip(1) {
                                if count % len == 0 {
                                    begin -= 1;
                                    len *= *num;
                                }
                                else {
                                    break;
                                }
                            }

                            scope.set_array_info(cur_array_info[begin..].to_vec());
                            result.append(&mut elem.create(program, scope, info));
                            count += len;
                        }
                    }
                }

                result.append(&mut vec![0; cur_array_info.iter().fold(1, |acc, num| acc * num) - count]);

                result
            }
        }
    }
}

impl<'ast> Create<'ast> for VarDecl {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        for def in &self.defs {
            def.create(program, scope, info);
        }
    }
}

impl<'ast> Create<'ast> for VarDef {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        if self.dims.is_empty() {
            // single variable
            if scope.is_global() {
                let init = self.init.as_ref().map_or(0, |val| {
                    match val {
                        InitVal::Exp(exp) => exp.evaluate(scope).unwrap(),
                        InitVal::List(_) => panic!("can't initialize a single variable with a list"),
                    }
                });
                let init = program.new_value().integer(init);
                let global_alloc = program.new_value().global_alloc(init);
                info.new_info(global_alloc.clone());
                scope.new_value(&self.id, Entry::Value(global_alloc));
            }
            else {
                match &self.init {
                    Some(init) => {
                        let value = match init {
                            InitVal::Exp(exp) => exp.create(program, scope, info),
                            InitVal::List(_) => panic!("can't initialize a single variable with a list"),
                        };

                        let alloc = new_value!(program, scope).alloc(Type::get_i32());
                        push_value!(program, scope, alloc.clone());
                        info.new_info(alloc.clone());

                        let store = new_value!(program, scope).store(value.clone(), alloc.clone());
                        push_value!(program, scope, store.clone());
                        info.new_info(store.clone());

                        info.info_mut(value).unwrap().death = info.counter();
                        info.info_mut(alloc).unwrap().death = info.counter();

                        scope.new_value(&self.id, Entry::Value(alloc));
                    }
                    None => {
                        let alloc = new_value!(program, scope).alloc(Type::get_i32());
                        push_value!(program, scope, alloc.clone());
                        info.new_info(alloc.clone());

                        scope.new_value(&self.id, Entry::Value(alloc));
                    }
                }
            }
        }
        else {
            // array
            let array_info: Vec<_> = self.dims.iter().map(|dim| dim.create(program, scope, info) as usize).collect();
            scope.set_array_info(array_info.clone());

            if scope.is_global() {
                let init = self.init.as_ref().map_or(vec![program.new_value().integer(0); array_info.iter().fold(1, |acc, num| acc * num)], |val| {
                    val.create(program, scope, info)
                });
                let init = global_array_init(program, init, array_info);
                let global_alloc = program.new_value().global_alloc(init);
                info.new_info(global_alloc.clone());
                scope.new_value(&self.id, Entry::Value(global_alloc));
            }
            else {
                let zero = new_value!(program, scope).integer(0);
                push_value!(program, scope, zero.clone());
                info.new_info(zero.clone());

                let values = self.init.as_ref().map_or(vec![zero; array_info.iter().fold(1, |acc, num| acc * num)], |val| {
                    val.create(program, scope, info)
                });

                let alloc = new_value!(program, scope).alloc(array_info.turn_into(()));
                push_value!(program, scope, alloc.clone());
                info.new_info(alloc.clone());

                local_array_init(program, scope, info, values, array_info, alloc.clone());

                scope.new_value(&self.id, Entry::Value(alloc));
            }
        }
    }
}

impl<'ast> Create<'ast> for InitVal {
    type Out = Vec<Value>;
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        if scope.array_info().is_empty() {
            return vec![];
        }

        let mut result = vec![];

        let cur_array_info = scope.array_info().clone();
        let base = *(cur_array_info.last().unwrap());
        let mut count = 0;

        let zero = if scope.is_global() {
            program.new_value().integer(0)
        }
        else {
            let zero = new_value!(program, scope).integer(0);
            push_value!(program, scope, zero.clone());
            info.new_info(zero.clone());
            zero
        };

        match self {
            Self::Exp(_) => panic!("can't initialize an array with a single expression"),
            Self::List(list) => {
                for elem in list.iter() {
                    match elem {
                        Self::Exp(exp) => {
                            result.push(exp.create(program, scope, info));
                            count += 1;
                        }
                        Self::List(_) => {
                            if count % base != 0 {
                                let remain = base - count % base;
                                result.append(&mut vec![zero.clone(); remain]);
                                count += remain;
                            }

                            let mut begin = cur_array_info.len() - 1;
                            let mut len = base;

                            for num in cur_array_info.iter().rev().skip(1) {
                                if count % len == 0 {
                                    begin -= 1;
                                    len *= *num;
                                }
                                else {
                                    break;
                                }
                            }

                            scope.set_array_info(cur_array_info[begin..].to_vec());
                            result.append(&mut elem.create(program, scope, info));
                            count += len;
                        }
                    }
                }

                result.append(&mut vec![zero.clone(); cur_array_info.iter().fold(1, |acc, num| acc * num) - count]);

                result
            }
        }
    }
}

impl<'ast> Create<'ast> for FuncDef {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        let name = format!("@{}", self.id);
        let params_ty: Vec<_> = self.params.iter().map(|param| param.create(program, scope, info)).collect();
        let ret_ty = self.ty.create(program, scope, info);

        let data = FunctionData::new(name, params_ty.clone(), ret_ty);
        let func = program.new_func(data);
        scope.new_func(&self.id, func.clone());
        scope.set_cur_func(Some(func.clone()));
        scope.enter();

        let mut count = 0;
        let data = program.func(func.clone());
        let params: Vec<_> = data.params().iter().map(|param| param.clone()).collect();
        for param in params {
            let alloc = new_value!(program, scope).alloc(params_ty[count].clone());
            push_value!(program, scope, alloc.clone());
            info.new_info(alloc.clone());

            let store = new_value!(program, scope).store(param.clone(), alloc.clone());
            push_value!(program, scope, store.clone());
            info.new_info(store.clone());

            info.info_mut(param.clone()).unwrap().death = info.counter();
            info.info_mut(alloc).unwrap().death = info.counter();

            scope.new_value(&self.params[count].id, Entry::Value(alloc));
            count += 1;
        }

        let entry = new_bb!(program, scope).basic_block(Some(format!("%entry")));
        push_bb!(program, scope, entry.clone());
        scope.set_cur_bb(Some(entry.clone()));

        self.body.create(program, scope, info);

        scope.exit();
        scope.set_cur_bb(None);
        scope.set_cur_func(None);
    }
}

impl<'ast> Create<'ast> for FuncParam {
    type Out = Type;
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        self.dims.as_ref().map_or(Type::get_i32(), |array_info| {
            let array_info: Vec<_> = array_info.iter().map(|dim| dim.create(program, scope, info) as usize).collect();
            Type::get_pointer(array_info.turn_into(()))
        })
    }
}

impl<'ast> Create<'ast> for FuncType {
    type Out = Type;
    fn create(&'ast self, _: &mut Program, _: &mut Scope<'ast>, _: &mut Info) -> Self::Out {
        match self {
            Self::Void => Type::get_unit(),
            Self::Int => Type::get_i32(),
        }
    }
}

impl<'ast> Create<'ast> for Block {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        scope.enter();
        for item in &self.items {
            item.create(program, scope, info);
        }
        scope.exit();
    }
}

impl<'ast> Create<'ast> for BlockItem {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        match self {
            Self::Decl(decl) => decl.create(program, scope, info),
            Self::Stmt(stmt) => stmt.create(program, scope, info),
        }
    }
}

impl<'ast> Create<'ast> for Stmt {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        match self {
            Self::Return(ret) => ret.create(program, scope, info),
            Self::Assign(asg) => asg.create(program, scope, info),
            Self::Exp(exp) => if let Some(exp) = exp {
                exp.create(program, scope, info);
            },
            Self::Block(blk) => blk.create(program, scope, info),
            Self::If(f) => f.create(program, scope, info),
            Self::While(whl) => whl.create(program, scope, info),
            Self::Break(brk) => brk.create(program, scope, info),
            Self::Continue(ctn) => ctn.create(program, scope, info),
        }
    }
}

impl<'ast> Create<'ast> for Return {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        let value = self.exp.as_ref().map_or(None, |exp| Some(exp.create(program, scope, info)));

        let ret = new_value!(program, scope).ret(value);
        push_value!(program, scope, ret.clone());
        info.new_info(ret.clone());

        if let Some(value) = value {
            info.info_mut(value).unwrap().death = info.counter();
        }
    }
}

impl<'ast> Create<'ast> for Assign {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        let mut dest = match scope.value(&self.lval.id) {
            Entry::Value(value) => value.clone(),
            Entry::Const(_) => panic!("have solved const"),
        };
        let value = self.exp.create(program, scope, info);
        
        // 0 presents undef
        // 1 presents a single variable
        // 2 presents an element in an array
        // 3 presnts an array or a slice
        // 4 presents a pointer(solved as case 3)
        let mut case = 0;
        if dest.is_global() {
            match program.borrow_value(dest.clone()).ty().kind() {
                TypeKind::Pointer(base) => {
                    match base.kind() {
                        TypeKind::Int32 => case = 1,
                        TypeKind::Array(_, _) => {
                            if self.lval.dims.len() != base.turn_into(()) {
                                case = 3;
                            }
                            else {
                                case = 2
                            }
                        }
                        TypeKind::Pointer(_) => case = 4,
                        _ => panic!("can't assign to a unit or function")
                    }
                }
                _ => panic!("can't assign to a value not created by alloc")
            }
        }
        else {
            match program.func(scope.cur_func().clone()).dfg().value(dest.clone()).ty().kind() {
                TypeKind::Pointer(base) => {
                    match base.kind() {
                        TypeKind::Int32 => case = 1,
                        TypeKind::Array(_, _) => {
                            if self.lval.dims.len() != base.turn_into(()) {
                                case = 3;
                            }
                            else {
                                case = 2
                            }
                        }
                        TypeKind::Pointer(_) => case = 4,
                        _ => panic!("can't assign to a unit or function")
                    }
                }
                _ => panic!("can't assign to a value not created by alloc")
            }
        }

        if case == 1 {
            let store = new_value!(program, scope).store(value.clone(), dest.clone());
            push_value!(program, scope, store.clone());
            info.new_info(store.clone());

            info.info_mut(value).unwrap().death = info.counter();
            info.info_mut(dest).unwrap().death = info.counter();
        }
        else if case == 2 {
            for dim in &self.lval.dims {
                let index = dim.create(program, scope, info);
                let get_elem_ptr = new_value!(program, scope).get_elem_ptr(dest.clone(), index.clone());
                push_value!(program, scope, get_elem_ptr.clone());
                info.new_info(get_elem_ptr.clone());

                info.info_mut(index).unwrap().death = info.counter();
                info.info_mut(dest.clone()).unwrap().death = info.counter();

                dest = get_elem_ptr;
            }

            let store = new_value!(program, scope).store(value.clone(), dest.clone());
            push_value!(program, scope, store.clone());
            info.new_info(store.clone());

            info.info_mut(value).unwrap().death = info.counter();
            info.info_mut(dest).unwrap().death = info.counter();
        }
        else {
            let load = new_value!(program, scope).load(dest.clone());
            push_value!(program, scope, load.clone());
            info.new_info(load.clone());

            info.info_mut(dest.clone()).unwrap().death = info.counter();

            dest = load;
            let mut first = true;

            for dim in &self.lval.dims {
                let index = dim.create(program, scope, info);
                let value = if first {
                    first = false;
                    new_value!(program, scope).get_ptr(dest.clone(), index.clone())
                }
                else {
                    new_value!(program, scope).get_elem_ptr(dest.clone(), index.clone())
                };
                push_value!(program, scope, value.clone());
                info.new_info(value.clone());

                info.info_mut(index).unwrap().death = info.counter();
                info.info_mut(dest.clone()).unwrap().death = info.counter();

                dest = value;
            }

            let store = new_value!(program, scope).store(value.clone(), dest.clone());
            push_value!(program, scope, store.clone());
            info.new_info(store.clone());

            info.info_mut(value).unwrap().death = info.counter();
            info.info_mut(dest).unwrap().death = info.counter();
        }
    }
}

impl<'ast> Create<'ast> for If {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        let cond = self.cond.create(program, scope, info);
        let (then_bb, else_bb, end_bb) = scope.label_mut().if_label();

        let then_bb = new_bb!(program, scope).basic_block(Some(then_bb));
        let else_bb = new_bb!(program, scope).basic_block(Some(else_bb));
        let end_bb = new_bb!(program, scope).basic_block(Some(end_bb));

        let branch = new_value!(program, scope).branch(cond.clone(), then_bb.clone(), match &self.els {
            Some(_) => else_bb.clone(),
            None => end_bb.clone(),
        });
        push_value!(program, scope, branch.clone());
        info.new_info(branch.clone());

        info.info_mut(cond).unwrap().death = info.counter();

        push_bb!(program, scope, then_bb.clone());
        scope.set_cur_bb(Some(then_bb.clone()));
        self.then.create(program, scope, info);

        let jump = new_value!(program, scope).jump(end_bb.clone());
        push_value!(program, scope, jump.clone());
        info.new_info(jump.clone());

        match &self.els {
            Some(els) => {
                push_bb!(program, scope, else_bb.clone());
                scope.set_cur_bb(Some(else_bb.clone()));
                els.create(program, scope, info);

                let jump = new_value!(program, scope).jump(end_bb.clone());
                push_value!(program, scope, jump.clone());
                info.new_info(jump.clone());
            }
            None => {}
        }

        push_bb!(program, scope, end_bb.clone());
        scope.set_cur_bb(Some(end_bb.clone()));
    }
}

impl<'ast> Create<'ast> for While {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        let (entry_bb, body_bb, end_bb) = scope.label_mut().while_label();

        let entry_bb = new_bb!(program, scope).basic_block(Some(entry_bb));
        let body_bb = new_bb!(program, scope).basic_block(Some(body_bb));
        let end_bb = new_bb!(program, scope).basic_block(Some(end_bb));

        let jump = new_value!(program, scope).jump(entry_bb.clone());
        push_value!(program, scope, jump.clone());
        info.new_info(jump.clone());

        push_bb!(program, scope, entry_bb.clone());
        scope.set_cur_bb(Some(entry_bb.clone()));

        let cond = self.cond.create(program, scope, info);
        let branch = new_value!(program, scope).branch(cond.clone(), body_bb.clone(), end_bb.clone());
        push_value!(program, scope, branch.clone());
        info.new_info(branch.clone());

        info.info_mut(cond).unwrap().death = info.counter();

        push_bb!(program, scope, body_bb.clone());
        scope.set_cur_bb(Some(body_bb.clone()));
        scope.loop_info_mut().push((entry_bb.clone(), end_bb.clone()));
        self.body.create(program, scope, info);

        let jump = new_value!(program, scope).jump(entry_bb.clone());
        push_value!(program, scope, jump.clone());
        info.new_info(jump.clone());

        scope.loop_info_mut().pop();
        push_bb!(program, scope, end_bb.clone());
        scope.set_cur_bb(Some(end_bb.clone()));
    }
}

impl<'ast> Create<'ast> for Break {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        let (_, end_bb) = scope.loop_info().last().unwrap();
        let jump = new_value!(program, scope).jump(end_bb.clone());
        push_value!(program, scope, jump.clone());
        info.new_info(jump.clone()); 
    }
}

impl<'ast> Create<'ast> for Continue {
    type Out = ();
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        let (entry_bb, _) = scope.loop_info().last().unwrap();
        let jump = new_value!(program, scope).jump(entry_bb.clone());
        push_value!(program, scope, jump.clone());
        info.new_info(jump.clone()); 
    }
}

impl<'ast> Create<'ast> for Exp {
    type Out = Value;
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        self.lor.create(program, scope, info)
    }
}

impl<'ast> Create<'ast> for LVal {
    type Out = Value;
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        if let Entry::Const(num) = scope.value(&self.id) {
            let integer = new_value!(program, scope).integer(*num);
            push_value!(program, scope, integer.clone());
            info.new_info(integer.clone());
            return integer;
        }
        let value = match scope.value(&self.id) {
            Entry::Value(value) => value,
            Entry::Const(_) => panic!("have solved const"),
        };
        
        let mut case = 0;
        if value.is_global() {
            match program.borrow_value(value.clone()).ty().kind() {
                TypeKind::Pointer(base) => {
                    match base.kind() {
                        TypeKind::Int32 => case = 1,
                        TypeKind::Array(_, _) => {
                            if self.dims.len() != base.turn_into(()) {
                                case = 3;
                            }
                            else {
                                case = 2
                            }
                        }
                        TypeKind::Pointer(ty) => {
                            if self.dims.len() != ty.turn_into(()) + 1 {
                                case = 5;
                            }
                            else {
                                case = 4;
                            }
                        }
                        _ => panic!("can't assign to a unit or function")
                    }
                }
                _ => panic!("can't assign to a value not created by alloc")
            }
        }
        else {
            match program.func(scope.cur_func().clone()).dfg().value(value.clone()).ty().kind() {
                TypeKind::Pointer(base) => {
                    match base.kind() {
                        TypeKind::Int32 => case = 1,
                        TypeKind::Array(_, _) => {
                            if self.dims.len() != base.turn_into(()) {
                                case = 3;
                            }
                            else {
                                case = 2
                            }
                        }
                        TypeKind::Pointer(ty) => {
                            if self.dims.len() != ty.turn_into(()) + 1 {
                                case = 5;
                            }
                            else {
                                case = 4;
                            }
                        }
                        _ => panic!("can't assign to a unit or function")
                    }
                }
                _ => panic!("can't assign to a value not created by alloc")
            }
        }

        if case == 1 {
            let load = new_value!(program, scope).load(value.clone());
            push_value!(program, scope, load.clone());
            info.new_info(load.clone());

            info.info_mut(value.clone()).unwrap().death = info.counter();

            load
        }
        else if case == 2 {
            let mut src = value.clone();
            for exp in &self.dims {
                let index = exp.create(program, scope, info);
                let get_elem_ptr = new_value!(program, scope).get_elem_ptr(src.clone(), index.clone());
                push_value!(program, scope, get_elem_ptr.clone());
                info.new_info(get_elem_ptr.clone());

                info.info_mut(index).unwrap().death = info.counter();
                info.info_mut(src.clone()).unwrap().death = info.counter();

                src = get_elem_ptr;
            }

            let load = new_value!(program, scope).load(src.clone());
            push_value!(program, scope, load.clone());
            info.new_info(load.clone());

            info.info_mut(src.clone()).unwrap().death = info.counter();

            load
        } 
        else if case == 3 {
            let mut src = value.clone();
            for exp in &self.dims {
                let index = exp.create(program, scope, info);
                let get_elem_ptr = new_value!(program, scope).get_elem_ptr(src.clone(), index.clone());
                push_value!(program, scope, get_elem_ptr.clone());
                info.new_info(get_elem_ptr.clone());

                info.info_mut(index).unwrap().death = info.counter();
                info.info_mut(src.clone()).unwrap().death = info.counter();

                src = get_elem_ptr;
            }

            let index = new_value!(program, scope).integer(0);
            push_value!(program, scope, index.clone());
            info.new_info(index.clone());

            let get_elem_ptr = new_value!(program, scope).get_elem_ptr(src.clone(), index.clone());
            push_value!(program, scope, get_elem_ptr.clone());
            info.new_info(get_elem_ptr.clone());

            info.info_mut(index.clone()).unwrap().death = info.counter();
            info.info_mut(src.clone()).unwrap().death = info.counter();

            get_elem_ptr
        }
        else if case == 4 {
            let load = new_value!(program, scope).load(value.clone());
            push_value!(program, scope, load.clone());
            info.new_info(load.clone());

            info.info_mut(value.clone()).unwrap().death = info.counter();

            let mut src = load;
            let mut first = true;
            
            for exp in &self.dims {
                let index = exp.create(program, scope, info);
                let value = if first {
                    first = false;
                    new_value!(program, scope).get_ptr(src.clone(), index.clone())
                }
                else {
                    new_value!(program, scope).get_elem_ptr(src.clone(), index.clone())
                };
                push_value!(program, scope, value.clone());
                info.new_info(value.clone());

                info.info_mut(index).unwrap().death = info.counter();
                info.info_mut(src.clone()).unwrap().death = info.counter();

                src = value;
            }

            let load = new_value!(program, scope).load(src.clone());
            push_value!(program, scope, load.clone());
            info.new_info(load.clone());

            info.info_mut(src.clone()).unwrap().death = info.counter();

            load
        }
        else {
            let load = new_value!(program, scope).load(value.clone());
            push_value!(program, scope, load.clone());
            info.new_info(load.clone());

            info.info_mut(value.clone()).unwrap().death = info.counter();

            let mut src = load;
            let mut first = true;
            
            for exp in &self.dims {
                let index = exp.create(program, scope, info);
                let value = if first {
                    first = false;
                    new_value!(program, scope).get_ptr(src.clone(), index.clone())
                }
                else {
                    new_value!(program, scope).get_elem_ptr(src.clone(), index.clone())
                };
                push_value!(program, scope, value.clone());
                info.new_info(value.clone());

                info.info_mut(index).unwrap().death = info.counter();
                info.info_mut(src.clone()).unwrap().death = info.counter();

                src = value;
            }

            let index = new_value!(program, scope).integer(0);
            push_value!(program, scope, index.clone());
            info.new_info(index.clone());

            if self.dims.len() > 0 {
                let get_elem_ptr = new_value!(program, scope).get_elem_ptr(src.clone(), index.clone());
                push_value!(program, scope, get_elem_ptr.clone());
                info.new_info(get_elem_ptr.clone());

                info.info_mut(index.clone()).unwrap().death = info.counter();
                info.info_mut(src.clone()).unwrap().death = info.counter();

                get_elem_ptr
            }
            else {
                src
            }
        }
    }
}

impl<'ast> Create<'ast> for PrimaryExp {
    type Out = Value;
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        match self {
            Self::Exp(exp) => exp.create(program, scope, info),
            Self::LVal(lval) => lval.create(program, scope, info),
            Self::Num(num) => {
                let integer = new_value!(program, scope).integer(*num);
                push_value!(program, scope, integer.clone());
                info.new_info(integer.clone());
                integer
            }
        }
    }
}

impl<'ast> Create<'ast> for UnaryExp {
    type Out = Value;
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        match self.evaluate(scope) {
            Some(num) => {
                let integer = new_value!(program, scope).integer(num);
                push_value!(program, scope, integer.clone());
                info.new_info(integer.clone());
                return integer;
            }
            None => {}
        }
        match self {
            Self::Primary(primary) => primary.create(program, scope, info),
            Self::Call(call) => call.create(program, scope, info),
            Self::Unary(op, unary) => {
                let rhs = unary.create(program, scope, info);
                if let UnaryOp::Pos = op {
                    return rhs;
                }
                let lhs = new_value!(program, scope).integer(0);
                push_value!(program, scope, lhs.clone());
                info.new_info(lhs.clone());

                let binary = new_value!(program, scope).binary(match op {
                    UnaryOp::Neg => BinaryOp::Sub,
                    UnaryOp::Not => BinaryOp::Eq,
                    _ => panic!("this op isn't defined in unary")
                }, lhs.clone(), rhs.clone());
                push_value!(program, scope, binary.clone());
                info.new_info(binary.clone());

                info.info_mut(lhs.clone()).unwrap().death = info.counter();
                info.info_mut(rhs.clone()).unwrap().death = info.counter();

                binary
            }
        }
    }
}

impl<'ast> Create<'ast> for Call {
    type Out = Value;
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        let args: Vec<_> = self.args.iter().map(|arg| arg.create(program, scope, info)).collect();
        let func = scope.func(&self.id);

        let call = new_value!(program, scope).call(func.clone(), args.clone());
        push_value!(program, scope, call.clone());
        info.new_info(call.clone());

        args.iter().for_each(|arg| info.info_mut(arg.clone()).unwrap().death = info.counter());
        call
    }
}

impl<'ast> Create<'ast> for MulExp {
    type Out = Value;
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        match self.evaluate(scope) {
            Some(num) => {
                let integer = new_value!(program, scope).integer(num);
                push_value!(program, scope, integer.clone());
                info.new_info(integer.clone());
                return integer;
            }
            None => {}
        }
        match self {
            Self::Unary(unary) => unary.create(program, scope, info),
            Self::Mul(mul, op, unary) => {
                let lhs = mul.create(program, scope, info);
                let rhs = unary.create(program, scope, info);

                let binary = new_value!(program, scope).binary(match op {
                    MulOp::Mul => BinaryOp::Mul,
                    MulOp::Div => BinaryOp::Div,
                    MulOp::Mod => BinaryOp::Mod,
                }, lhs.clone(), rhs.clone());
                push_value!(program, scope, binary.clone());
                info.new_info(binary.clone());

                info.info_mut(lhs.clone()).unwrap().death = info.counter();
                info.info_mut(rhs.clone()).unwrap().death = info.counter();

                binary
            }
        }
    }
}

impl<'ast> Create<'ast> for AddExp {
    type Out = Value;
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out { 
        match self.evaluate(scope) {
            Some(num) => {
                let integer = new_value!(program, scope).integer(num);
                push_value!(program, scope, integer.clone());
                info.new_info(integer.clone());
                return integer;
            }
            None => {}
        }
        match self {
            Self::Mul(mul) => mul.create(program, scope, info),
            Self::Add(add, op, mul) => {
                let lhs = add.create(program, scope, info);
                let rhs = mul.create(program, scope, info);

                let binary = new_value!(program, scope).binary(match op {
                    AddOp::Add => BinaryOp::Add,
                    AddOp::Sub => BinaryOp::Sub,
                }, lhs.clone(), rhs.clone());
                push_value!(program, scope, binary.clone());
                info.new_info(binary.clone());

                info.info_mut(lhs.clone()).unwrap().death = info.counter();
                info.info_mut(rhs.clone()).unwrap().death = info.counter();

                binary
            }
        }
    }
}

impl<'ast> Create<'ast> for RelExp {
    type Out = Value;
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out { 
        match self.evaluate(scope) {
            Some(num) => {
                let integer = new_value!(program, scope).integer(num);
                push_value!(program, scope, integer.clone());
                info.new_info(integer.clone());
                return integer;
            }
            None => {}
        }
        match self {
            Self::Add(add) => add.create(program, scope, info),
            Self::Rel(rel, op, add) => {
                let lhs = rel.create(program, scope, info);
                let rhs = add.create(program, scope, info);

                let binary = new_value!(program, scope).binary(match op {
                    RelOp::Lt => BinaryOp::Lt,
                    RelOp::Gt => BinaryOp::Gt,
                    RelOp::Le => BinaryOp::Le,
                    RelOp::Ge => BinaryOp::Ge,
                }, lhs.clone(), rhs.clone());
                push_value!(program, scope, binary.clone());
                info.new_info(binary.clone());

                info.info_mut(lhs.clone()).unwrap().death = info.counter();
                info.info_mut(rhs.clone()).unwrap().death = info.counter();

                binary
            }
        }
    }
}

impl<'ast> Create<'ast> for EqExp {
    type Out = Value;
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out { 
        match self.evaluate(scope) {
            Some(num) => {
                let integer = new_value!(program, scope).integer(num);
                push_value!(program, scope, integer.clone());
                info.new_info(integer.clone());
                return integer;
            }
            None => {}
        }
        match self {
            Self::Rel(rel) => rel.create(program, scope, info),
            Self::Eq(eq, op, rel) => {
                let lhs = eq.create(program, scope, info);
                let rhs = rel.create(program, scope, info);

                let binary = new_value!(program, scope).binary(match op {
                    EqOp::Eq => BinaryOp::Eq,
                    EqOp::Ne => BinaryOp::NotEq,
                }, lhs.clone(), rhs.clone());
                push_value!(program, scope, binary.clone());
                info.new_info(binary.clone());

                info.info_mut(lhs.clone()).unwrap().death = info.counter();
                info.info_mut(rhs.clone()).unwrap().death = info.counter();

                binary
            }
        }
    }
}

impl<'ast> Create<'ast> for LAndExp {
    type Out = Value;
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        match self.evaluate(scope) {
            Some(num) => {
                let integer = new_value!(program, scope).integer(num);
                push_value!(program, scope, integer.clone());
                info.new_info(integer.clone());
                return integer;
            }
            None => {}
        }
        match self {
            Self::Eq(eq) => eq.create(program, scope, info),
            Self::LAnd(land, eq) => {
                let lhs = land.create(program, scope, info);
                let (then_bb, _, end_bb) = scope.label_mut().if_label();

                let then_bb = new_bb!(program, scope).basic_block(Some(then_bb));
                let end_bb = new_bb!(program, scope).basic_block(Some(end_bb));

                let alloc = new_value!(program, scope).alloc(Type::get_i32());
                push_value!(program, scope, alloc.clone());
                info.new_info(alloc.clone());

                let zero = new_value!(program, scope).integer(0);
                push_value!(program, scope, zero.clone());
                info.new_info(zero.clone());

                let store = new_value!(program, scope).store(zero.clone(), alloc.clone());
                push_value!(program, scope, store.clone());
                info.new_info(store.clone());

                info.info_mut(zero.clone()).unwrap().death = info.counter();
                info.info_mut(alloc.clone()).unwrap().death = info.counter();

                let cond = new_value!(program, scope).binary(BinaryOp::NotEq, lhs.clone(), zero.clone());
                push_value!(program, scope, cond.clone());
                info.new_info(cond.clone());

                info.info_mut(lhs.clone()).unwrap().death = info.counter();
                info.info_mut(zero.clone()).unwrap().death = info.counter();

                let branch = new_value!(program, scope).branch(cond.clone(), then_bb.clone(), end_bb.clone());
                push_value!(program, scope, branch.clone());
                info.new_info(branch.clone());

                info.info_mut(cond.clone()).unwrap().death = info.counter();

                push_bb!(program, scope, then_bb.clone());
                scope.set_cur_bb(Some(then_bb.clone()));

                let rhs = eq.create(program, scope, info);
                let result = new_value!(program, scope).binary(BinaryOp::NotEq, rhs.clone(), zero.clone());
                push_value!(program, scope, result.clone());
                info.new_info(result.clone());

                info.info_mut(rhs.clone()).unwrap().death = info.counter();
                info.info_mut(zero.clone()).unwrap().death = info.counter();

                let store = new_value!(program, scope).store(result.clone(), alloc.clone());
                push_value!(program, scope, store.clone());
                info.new_info(store.clone());

                info.info_mut(result.clone()).unwrap().death = info.counter();
                info.info_mut(alloc.clone()).unwrap().death = info.counter();

                let jump = new_value!(program, scope).jump(end_bb.clone());
                push_value!(program, scope, jump.clone());
                info.new_info(jump.clone());

                push_bb!(program, scope, end_bb.clone());
                scope.set_cur_bb(Some(end_bb.clone()));

                let load = new_value!(program, scope).load(alloc.clone());
                push_value!(program, scope, load.clone());
                info.new_info(load.clone());

                info.info_mut(alloc.clone()).unwrap().death = info.counter();

                load
            }
        }
    }
}

impl<'ast> Create<'ast> for LOrExp {
    type Out = Value;
    fn create(&'ast self, program: &mut Program, scope: &mut Scope<'ast>, info: &mut Info) -> Self::Out {
        match self.evaluate(scope) {
            Some(num) => {
                let integer = new_value!(program, scope).integer(num);
                push_value!(program, scope, integer.clone());
                info.new_info(integer.clone());
                return integer;
            }
            None => {}
        }
        match self {
            Self::LAnd(land) => land.create(program, scope, info),
            Self::LOr(lor, land) => {
                let lhs = lor.create(program, scope, info);
                let (then_bb, _, end_bb) = scope.label_mut().if_label();

                let then_bb = new_bb!(program, scope).basic_block(Some(then_bb));
                let end_bb = new_bb!(program, scope).basic_block(Some(end_bb));

                let alloc = new_value!(program, scope).alloc(Type::get_i32());
                push_value!(program, scope, alloc.clone());
                info.new_info(alloc.clone());

                let zero = new_value!(program, scope).integer(0);
                push_value!(program, scope, zero.clone());
                info.new_info(zero.clone());

                let one = new_value!(program, scope).integer(1);
                push_value!(program, scope, one.clone());
                info.new_info(one.clone());

                let store = new_value!(program, scope).store(one.clone(), alloc.clone());
                push_value!(program, scope, store.clone());
                info.new_info(store.clone());

                info.info_mut(one.clone()).unwrap().death = info.counter();
                info.info_mut(alloc.clone()).unwrap().death = info.counter();

                let cond = new_value!(program, scope).binary(BinaryOp::Eq, lhs.clone(), zero.clone());
                push_value!(program, scope, cond.clone());
                info.new_info(cond.clone());

                info.info_mut(lhs.clone()).unwrap().death = info.counter();
                info.info_mut(zero.clone()).unwrap().death = info.counter();

                let branch = new_value!(program, scope).branch(cond.clone(), then_bb.clone(), end_bb.clone());
                push_value!(program, scope, branch.clone());
                info.new_info(branch.clone());

                info.info_mut(cond.clone()).unwrap().death = info.counter();

                push_bb!(program, scope, then_bb.clone());
                scope.set_cur_bb(Some(then_bb.clone()));

                let rhs = land.create(program, scope, info);
                let result = new_value!(program, scope).binary(BinaryOp::NotEq, rhs.clone(), zero.clone());
                push_value!(program, scope, result.clone());
                info.new_info(result.clone());

                info.info_mut(rhs.clone()).unwrap().death = info.counter();
                info.info_mut(zero.clone()).unwrap().death = info.counter();

                let store = new_value!(program, scope).store(result.clone(), alloc.clone());
                push_value!(program, scope, store.clone());
                info.new_info(store.clone());

                info.info_mut(result.clone()).unwrap().death = info.counter();
                info.info_mut(alloc.clone()).unwrap().death = info.counter();

                let jump = new_value!(program, scope).jump(end_bb.clone());
                push_value!(program, scope, jump.clone());
                info.new_info(jump.clone());

                push_bb!(program, scope, end_bb.clone());
                scope.set_cur_bb(Some(end_bb.clone()));

                let load = new_value!(program, scope).load(alloc.clone());
                push_value!(program, scope, load.clone());
                info.new_info(load.clone());

                info.info_mut(alloc.clone()).unwrap().death = info.counter();

                load
            }
        }
    }
}

impl<'ast> Create<'ast> for ConstExp {
    type Out = i32;
    fn create(&self, _: &mut Program, scope: &mut Scope<'ast>, _: &mut Info) -> Self::Out {
        match self.exp.evaluate(scope) {
            Some(num) => num,
            None => panic!("can't evaluate constant expression"),
        }
    }
}
