//! # Tools
//! 
//! This module defines some tool functions used in other modules.
//! 

use crate::mem::scope::{ Scope, new_value, push_value };
use crate::mem::info::Info;
use koopa::ir::{ Program, Value, Type, TypeKind };
use koopa::ir::builder_traits::*;

/// Initialize a global const array. Return aggregate value.
pub fn global_const_array_init(program: &mut Program, nums: Vec<i32>, array_info: Vec<usize>) -> Value {
    if array_info.is_empty() {
        program.new_value().integer(nums[0])
    }
    else {
        let mut next_nums = vec![];
        let next_array_info = array_info[1..].to_vec();

        let mut elems = vec![];
        let elem_len = next_array_info.iter().fold(1, |acc, &x| acc * x);

        for i in 0..nums.len() {
            next_nums.push(nums[i]);
            if (i + 1) % elem_len == 0 {
                elems.push(global_const_array_init(program, next_nums, next_array_info.clone()));
                next_nums = vec![];
            }
        }

        program.new_value().aggregate(elems)
    }
}

/// Initialize a local const array. No return.
pub fn local_const_array_init(program: &mut Program, scope: &mut Scope, info: &mut Info, nums: Vec<i32>, array_info: Vec<usize>, dest: Value) {
    if array_info.is_empty() {
        let value = new_value!(program, scope).integer(nums[0]);
        push_value!(program, scope, value.clone());
        info.new_info(value.clone());

        let store = new_value!(program, scope).store(value.clone(), dest.clone());
        push_value!(program, scope, store.clone());
        info.new_info(store.clone());

        info.info_mut(value).unwrap().death = info.counter();
        info.info_mut(dest).unwrap().death = info.counter();
    }
    else {
        let mut next_nums = vec![];
        let next_array_info = array_info[1..].to_vec();

        let elem_len = next_array_info.iter().fold(1, |acc, &x| acc * x);

        for i in 0..nums.len() {
            next_nums.push(nums[i]);
            if (i + 1) % elem_len == 0 {
                let index = new_value!(program, scope).integer((i / elem_len) as i32);
                push_value!(program, scope, index.clone());
                info.new_info(index.clone());

                let get_elem_ptr = new_value!(program, scope).get_elem_ptr(dest.clone(), index.clone());
                push_value!(program, scope, get_elem_ptr.clone());
                info.new_info(get_elem_ptr.clone());

                info.info_mut(index).unwrap().death = info.counter();
                info.info_mut(dest.clone()).unwrap().death = info.counter();

                local_const_array_init(program, scope, info, next_nums, next_array_info.clone(), get_elem_ptr);
                next_nums = vec![];
            }
        }
    }
}

/// Initialize a global array. Return aggregate value.
pub fn global_array_init(program: &mut Program, values: Vec<Value>, array_info: Vec<usize>) -> Value {
    if array_info.is_empty() {
        values[0].clone()
    }
    else {
        let mut next_values = vec![];
        let next_array_info = array_info[1..].to_vec();

        let mut elems = vec![];
        let elem_len = next_array_info.iter().fold(1, |acc, &x| acc * x);



        for i in 0..values.len() {
            next_values.push(values[i]);
            if (i + 1) % elem_len == 0 {
                elems.push(global_array_init(program, next_values, next_array_info.clone()));
                next_values = vec![];
            }
        }

        program.new_value().aggregate(elems)
    }
}

pub fn local_array_init(program: &mut Program, scope: &mut Scope, info: &mut Info, values: Vec<Value>, array_info: Vec<usize>, dest: Value) {
    if array_info.is_empty() {
        let value = values[0].clone();
        let store = new_value!(program, scope).store(value.clone(), dest.clone());
        push_value!(program, scope, store);
        info.new_info(store);

        info.info_mut(value).unwrap().death = info.counter();
        info.info_mut(dest).unwrap().death = info.counter();
    }
    else {
        let mut next_values = vec![];
        let next_array_info = array_info[1..].to_vec();

        let elem_len = next_array_info.iter().fold(1, |acc, &x| acc * x);

        for i in 0..values.len() {
            next_values.push(values[i].clone());
            if (i + 1) % elem_len == 0 {
                let index = new_value!(program, scope).integer((i / elem_len) as i32);
                push_value!(program, scope, index.clone());
                info.new_info(index.clone());

                let get_elem_ptr = new_value!(program, scope).get_elem_ptr(dest.clone(), index.clone());
                push_value!(program, scope, get_elem_ptr.clone());
                info.new_info(get_elem_ptr.clone());

                info.info_mut(index).unwrap().death = info.counter();
                info.info_mut(dest.clone()).unwrap().death = info.counter();

                local_array_init(program, scope, info, next_values, next_array_info.clone(), get_elem_ptr);
                next_values = vec![];
            }
        }
    }
}

/// My own Into trait for type conversion in this project.
pub trait TurnInto<T> {
    type Addition;
    fn turn_into(&self, addition: Self::Addition) -> T;
}

impl TurnInto<Type> for Vec<usize> {
    type Addition = ();
    fn turn_into(&self, _: Self::Addition) -> Type {
        if self.is_empty() {
            Type::get_i32()
        }
        else {
            Type::get_array(self[1..].to_vec().turn_into(()), self[0])
        }
    }
}

impl TurnInto<usize> for Type {
    type Addition = ();
    fn turn_into(&self, _: Self::Addition) -> usize {
        match self.kind() {
            TypeKind::Int32 => 0,
            TypeKind::Array(ty, _) => 1 + ty.turn_into(()),
            _ => panic!("Type error!"),
        }
    }
}

pub fn get_size_form_ty(ty: &Type) -> usize {
    match ty.kind() {
        TypeKind::Int32 => 1,
        TypeKind::Array(ty, len) => len * get_size_form_ty(ty),
        _ => panic!("we only expect int32 and array type"),
    }
}
