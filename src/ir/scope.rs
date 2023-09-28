//! # Scope
//! 
//! The scope manages all registers and labels. Besides, it documents the current state of the program.
//! 

use crate::tools::TurnInto;
use crate::ir::label::Label;
use std::collections::HashMap;
use koopa::ir::{ Function, Value };

#[derive(Clone)]
pub enum Entry {
    Register(usize),
    Label(String),
}

impl TurnInto<String> for Entry {
    type Addition = ();
    fn turn_into(&self, _: Self::Addition) -> String {
        match self {
            Entry::Register(reg) => format!("%{}", reg),
            Entry::Label(label) => label.clone(),
        }
    }
}

macro_rules! value {
    ($program: expr, $scope: expr, $value: expr) => {
        $program.func($scope.cur_func().clone())
            .dfg()
            .value($value)
    };
}
pub(crate) use value;

pub struct Scope {
    values: HashMap<Value, Entry>,
    register: usize,

    cur_func: Option<Function>,
    cur_value: Option<Value>,

    label: Label,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            register: 0,

            cur_func: None,
            cur_value: None,

            label: Label::new(),
        }
    }

    pub fn cur_func(&self) -> &Function{
        match self.cur_func {
            Some(ref func) => func,
            None => panic!("no function"),
        }
    }

    pub fn set_cur_func(&mut self, func: Option<Function>) {
        self.cur_func = func;
    }

    pub fn cur_value(&self) -> &Value {
        match self.cur_value {
            Some(ref value) => value,
            None => panic!("no value"),
        }
    }

    pub fn set_cur_value(&mut self, value: Option<Value>) {
        self.cur_value = value;
    }

    pub fn new_register(&mut self) -> Entry {
        let result = self.register;
        self.register += 1;
        Entry::Register(result)
    }

    pub fn clear_register(&mut self) {
        self.register = 0;
    }

    pub fn value(&self, value: &Value) -> &Entry {
        self.values.get(value).unwrap()
    }

    pub fn new_value(&mut self, value: Value, entry: Entry) {
        self.values.insert(value, entry);
    }

    pub fn label_mut(&mut self) -> &mut Label {
        &mut self.label
    }
}
