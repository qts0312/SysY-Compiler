//! # Scope
//! 
//! In this file, we define structure manages values and statement of the program.
//! 

use crate::asm::register::{ Registers, Register };
use crate::asm::label::Label;
use crate::asm::writer::Writer;
use std::collections::HashMap;
use koopa::ir::{ Value, Function };

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Entry {
    Slot(usize),    // slot in stack
    Register(String),   // register
    Label(String),  // label(data in heap)
}

macro_rules! new_register {
    ($scope: expr, $w: expr) => {
        {
            let register = $scope.new_register($w);
            let name = register.name().to_string();
            let value = register.value.clone();

            $scope.spill(value);

            let register = $scope.register_mut(&name);
            register
        }
    };
}
pub(crate) use new_register;

pub struct Scope {
    values: HashMap<Value, Entry>,

    cur_func: Option<Function>,
    cur_value: Option<Value>,

    registers: Registers,

    total_slots: usize,
    used_slots: usize,

    caller: bool,

    label: Label,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            cur_func: None,
            cur_value: None,
            registers: Registers::new(),
            total_slots: 0,
            used_slots: 0,
            caller: false,
            label: Label::new(),
        }
    }

    pub fn value(&self, value: &Value) -> &Entry {
        self.values.get(value).unwrap()
    }

    pub fn value_mut(&mut self, value: &Value) -> &mut Entry {
        self.values.get_mut(value).unwrap()
    }

    pub fn new_value(&mut self, value: Value, entry: Entry) {
        self.values.insert(value, entry);
    }

    pub fn cur_func(&self) -> &Function {
        self.cur_func.as_ref().unwrap()
    }

    pub fn set_cur_func(&mut self, func: Option<Function>) {
        self.cur_func = func;
    }

    pub fn cur_value(&self) -> &Value {
        self.cur_value.as_ref().unwrap()
    }

    pub fn set_cur_value(&mut self, value: Option<Value>) {
        self.cur_value = value;
    }

    pub fn caller(&self) -> bool {
        self.caller
    }

    pub fn set_caller(&mut self, caller: bool) {
        self.caller = caller;
    }

    pub fn label_mut(&mut self) -> &mut Label {
        &mut self.label
    }

    pub fn total_slots(&self) -> usize {
        self.total_slots
    }

    pub fn set_total_slots(&mut self, total_slots: usize) {
        self.total_slots = total_slots;
    }

    pub fn set_used_slots(&mut self, used_slots: usize) {
        self.used_slots = used_slots;
    }

    pub fn new_slot(&mut self) -> usize {
        let slot = self.used_slots;
        self.used_slots += 1;
        slot
    }

    pub fn new_slots(&mut self, slots: usize) -> usize {
        let slot = self.used_slots;
        self.used_slots += slots;
        slot
    }

    pub fn register(&mut self, id: &str) -> &Register {
        self.registers.register(id)
    }

    pub fn register_mut(&mut self, id: &str) -> &mut Register {
        self.registers.register_mut(id)
    }

    pub fn registers_mut(&mut self) -> &mut [Register] {
        self.registers.registers_mut()
    }

    pub fn new_register(&mut self, w: &mut Writer) -> &Register {
        let slot = self.used_slots;
        let unused = self.registers.registers().iter().find(|r| !r.used && !r.fixed);
        let oldest = self.registers.registers().iter().filter(|r| !r.fixed).min_by_key(|r| r.stamp);

        match (unused, oldest) {
            (Some(unused), _) => unused,
            (_, Some(oldest)) => {
                w.op2("sw", oldest.name(), &format!("{}(sp)", slot * 4));
                oldest
            }
            _ => panic!("all registers fixed"),
        }
    }

    pub fn spill(&mut self, value: Option<Value>) {
        match value {
            Some(value) => {
                self.new_value(value, Entry::Slot(self.used_slots));
                self.used_slots += 1;
            }
            None => {}
        }
    }
}
