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
}

impl Info {
    pub fn new() -> Self {
        Info {
            counter: 0,
            value_infos: HashMap::new(),
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
}
