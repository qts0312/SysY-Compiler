//! # Label
//! 
//! In this file, generator can get labels for global data.
//! 

pub struct Label {
    var: usize,
    arr: usize,
}

impl Label {
    pub fn new() -> Self {
        Self {
            var: 0,
            arr: 0,
        }
    }

    pub fn var(&mut self) -> String {
        let label = format!("var_{}", self.var);
        self.var += 1;
        label
    }

    pub fn arr(&mut self) -> String {
        let label = format!("arr_{}", self.arr);
        self.arr += 1;
        label
    }
}
