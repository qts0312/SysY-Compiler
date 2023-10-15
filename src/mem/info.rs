//! # Info
//! 
//! In this file, we define the structure to document the information for assembly generation.
//! 

use std::collections::HashMap;
use koopa::ir::Value;

pub struct ValueInfo {
    pub birth: usize,
    pub death: usize,
}

pub struct Info {
    counter: usize,
    value_infos: HashMap<Value, ValueInfo>,
    zero_array_infos: HashMap<Value, Vec<usize>>,
}

impl Info {
    pub fn new() -> Self {
        Info {
            counter: 0,
            value_infos: HashMap::new(),
            zero_array_infos: HashMap::new(),
        }
    }

    pub fn counter(&self) -> usize {
        self.counter
    }

    pub fn new_info(&mut self, value: Value) {
        self.value_infos.insert(value, ValueInfo {
            birth: self.counter,
            death: self.counter
        });
        self.counter += 1;
    }

    pub fn info(&self, value: Value) -> Option<&ValueInfo> {
        self.value_infos.get(&value)
    }

    pub fn info_mut(&mut self, value: Value) -> Option<&mut ValueInfo> {
        self.value_infos.get_mut(&value)
    }

    pub fn new_array_info(&mut self, value: Value, info: Vec<usize>) {
        self.zero_array_infos.insert(value, info);
    }

    pub fn array_info(&self, value: Value) -> Option<&Vec<usize>> {
        self.zero_array_infos.get(&value)
    }
}
