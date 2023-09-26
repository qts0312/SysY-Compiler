//! # Scope
//! 
//! This scope is a complex structure that contains all the information needed to generate IR.
//! * project from string to value.
//! * position of current value.
//! * information of loops.
//! * information of arrays.
//! * label generator.
//! 

use crate::mem::label::Label;
use std::collections::HashMap;
use koopa::ir::{ Value, Function, BasicBlock };

pub enum Entry {
    Const(i32),
    Value(Value),
}

macro_rules! new_value {
    ($program: expr, $scope: expr) => {
        $program.func_mut($scope.cur_func().clone())
            .dfg_mut()
            .new_value()
    };
}
pub(crate) use new_value;

macro_rules! push_value {
    ($program: expr, $scope: expr, $value: expr) => {
        let _ = $program.func_mut($scope.cur_func().clone())
            .layout_mut()
            .bb_mut($scope.cur_bb().clone())
            .insts_mut()
            .push_key_back($value);
    };
}
pub(crate) use push_value;

macro_rules! new_bb {
    ($program: expr, $scope: expr) => {
        $program.func_mut($scope.cur_func().clone())
            .dfg_mut()
            .new_bb()
    };
}
pub(crate) use new_bb;

macro_rules! push_bb {
    ($program: expr, $scope: expr, $bb: expr) => {
        let _ = $program.func_mut($scope.cur_func().clone())
            .layout_mut()
            .bbs_mut()
            .push_key_back($bb);
    };
}
pub(crate) use push_bb;

pub struct Scope<'ast> {
    values: Vec<HashMap<&'ast str, Entry>>,
    funcs: HashMap<&'ast str, Function>,

    cur_func: Option<Function>,
    cur_bb: Option<BasicBlock>,

    loop_info: Vec<(BasicBlock, BasicBlock)>,
    array_info: Vec<usize>,

    label: Label,
}

impl<'ast> Scope<'ast> {
    pub fn new() -> Self {
        Self {
            values: vec![HashMap::new()],
            funcs: HashMap::new(),

            cur_func: None,
            cur_bb: None,

            loop_info: Vec::new(),
            array_info: Vec::new(),

            label: Label::new(),
        }
    }

    pub fn value(&self, id: &'ast str) -> &Entry {
        for scope in self.values.iter().rev() {
            if let Some(entry) = scope.get(id) {
                return entry;
            }
        }
        panic!("value {} not found", id);
    }

    pub fn new_value(&mut self, id: &'ast str, entry: Entry) {
        self.values.last_mut().unwrap().insert(id, entry);
    }

    pub fn enter(&mut self) {
        self.values.push(HashMap::new());
    }

    pub fn exit(&mut self) {
        self.values.pop();
    }

    pub fn func(&self, id: &'ast str) -> &Function {
        match self.funcs.get(id) {
            Some(func) => func,
            None => panic!("function {} not found", id),
        }
    }

    pub fn new_func(&mut self, id: &'ast str, func: Function) {
        self.funcs.insert(id, func);
    }

    pub fn cur_func(&self) -> &Function {
        match &self.cur_func {
            Some(func) => func,
            None => panic!("no function"),
        }
    }

    pub fn set_cur_func(&mut self, func: Option<Function>) {
        self.cur_func = func;
    }

    pub fn is_global(&self) -> bool {
        match self.cur_func {
            Some(_) => false,
            None => true,
        }
    }

    pub fn cur_bb(&self) -> &BasicBlock {
        match &self.cur_bb {
            Some(bb) => bb,
            None => panic!("no basic block"),
        }
    }

    pub fn set_cur_bb(&mut self, bb: Option<BasicBlock>) {
        self.cur_bb = bb;
    }

    pub fn loop_info(&self) -> &Vec<(BasicBlock, BasicBlock)> {
        &self.loop_info
    }

    pub fn loop_info_mut(&mut self) -> &mut Vec<(BasicBlock, BasicBlock)> {
        &mut self.loop_info
    }

    pub fn array_info(&self) -> &Vec<usize> {
        &self.array_info
    }

    pub fn set_array_info(&mut self, array_info: Vec<usize>) {
        self.array_info = array_info;
    }

    pub fn label_mut(&mut self) -> &mut Label {
        &mut self.label
    }
}
