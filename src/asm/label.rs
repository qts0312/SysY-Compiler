//! # Label
//! 
//! In this file, generator can get labels for global data.
//! 

pub struct Label {
    var: usize,
}

impl Label {
    pub fn new() -> Self {
        Self {
            var: 0,
        }
    }

    pub fn var(&mut self) -> String {
        let label = format!("var_{}", self.var);
        self.var += 1;
        label
    }
}
