//! # Label
//! 
//! In this file, we define a label generator. 
//! 

pub struct Label {
    if_counter: usize,
    while_counter: usize,
}

impl Label {
    pub fn new() -> Self {
        Label {
            if_counter: 0,
            while_counter: 0,
        }
    }

    pub fn if_label(&mut self) -> (String, String, String) {
        let counter = self.if_counter;
        self.if_counter += 1;
        (
            format!("%Then_{}", counter),
            format!("%Else_{}", counter),
            format!("%End_{}", counter),
        )
    }

    pub fn while_label(&mut self) -> (String, String, String) {
        let counter = self.while_counter;
        self.while_counter += 1;
        (
            format!("%Entry_{}", counter),
            format!("%Body_{}", counter),
            format!("%While_End_{}", counter),
        )
    }
}
