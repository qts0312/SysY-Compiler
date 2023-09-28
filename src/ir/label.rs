//! # Label
//! 
//! This file defines a structure to allocate label for local and global, single variable and array.
//! 

pub struct Label {
    local_var: usize,
    local_arr: usize,
    global_var: usize,
    global_arr: usize,
}

impl Label {
    pub fn new() -> Self {
        Self {
            local_var: 0,
            local_arr: 0,
            global_var: 0,
            global_arr: 0,
        }
    }

    pub fn local_var(&mut self) -> String {
        let result = format!("@var{}", self.local_var);
        self.local_var += 1;
        result
    }

    pub fn local_arr(&mut self) -> String {
        let result = format!("@arr{}", self.local_arr);
        self.local_arr += 1;
        result
    }

    pub fn global_var(&mut self) -> String {
        let result = format!("@Gvar{}", self.global_var);
        self.global_var += 1;
        result
    }

    pub fn global_arr(&mut self) -> String {
        let result = format!("@Garr{}", self.global_arr);
        self.global_arr += 1;
        result
    }
}
