//! # Register
//! 
//! In this file, we define structure to manage temporary registers.
//! 

use koopa::ir::Value;

pub struct Register {
    pub name: String,
    pub used: bool,
    pub fixed: bool,    // this member may be useless
    pub stamp: usize,
    pub value: Option<Value>,
}

impl Register {
    pub fn new(name: String) -> Self {
        Self {
            name,
            used: false,
            fixed: false,
            stamp: 0,
            value: None,
        }
    }

    pub fn with_fixed(name: String) -> Self {
        Self {
            name,
            used: false,
            fixed: true,
            stamp: 0,
            value: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub struct Registers {
    pub registers: Vec<Register>,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            registers: vec![
                Register::with_fixed("x0".to_string()),
                Register::new("t0".to_string()),
                Register::new("t1".to_string()),
                Register::new("t2".to_string()),
                Register::new("t3".to_string()),
                Register::new("t4".to_string()),
                Register::new("t5".to_string()),
                Register::with_fixed("t6".to_string()),
            ],
        }
    }

    pub fn register(&self, name: &str) -> &Register {
        self.registers
            .iter()
            .find(|r| r.name() == name)
            .expect(&format!("Register {} not found", name))
    }

    pub fn register_mut(&mut self, name: &str) -> &mut Register {
        self.registers
            .iter_mut()
            .find(|r| r.name() == name)
            .expect(&format!("Register {} not found", name))
    }

    pub fn registers(&self) -> &[Register] {
        &self.registers
    }

    pub fn registers_mut(&mut self) -> &mut [Register] {
        &mut self.registers
    }
}
