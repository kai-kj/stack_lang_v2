use {
    crate::session::{BlockId, BuiltinId},
    std::rc::Rc,
};

pub type LValue = Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Integer(i64),
    String(Rc<str>),
    Block(BlockId),
    Builtin(BuiltinId),
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Integer(i) => *i != 0,
            Value::String(s) => !s.is_empty(),
            Value::Block(_) => true,
            Value::Builtin(_) => true,
        }
    }
}
