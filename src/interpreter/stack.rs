use crate::interpreter::{error::InterpretError, value::Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Stack {
    values: Vec<Value>,
}

impl Stack {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn push(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn pop(&mut self) -> Result<Value, InterpretError> {
        self.values.pop().ok_or(InterpretError::StackIsEmpty)
    }
}
