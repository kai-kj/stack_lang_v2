use crate::{
    location::Located,
    session::{BlockId, BuiltinId, Session, StringId},
};

pub type LHighOp = Located<HighOp>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HighOp {
    Call,
    CallConditional,
    PushBoolean(bool),
    PushInteger(i64),
    // PushFloat(f64),
    PushString(StringId),
    PushBlock(BlockId),
    PushBuiltin(BuiltinId),
}

impl HighOp {
    pub fn into_owned(self, sesh: &Session) -> OwnedHighOp {
        match self {
            HighOp::Call => OwnedHighOp::Call,
            HighOp::CallConditional => OwnedHighOp::CallConditional,
            HighOp::PushBoolean(v) => OwnedHighOp::PushBoolean(v),
            HighOp::PushInteger(v) => OwnedHighOp::PushInteger(v),
            HighOp::PushString(id) => OwnedHighOp::PushString(sesh.strings.get(id).to_string()),
            HighOp::PushBlock(id) => OwnedHighOp::PushBlock(
                sesh.blocks.get(id).into_iter().map(|op| op.val.into_owned(sesh)).collect(),
            ),
            HighOp::PushBuiltin(_) => OwnedHighOp::PushBuiltin,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OwnedHighOp {
    Call,
    CallConditional,
    PushBoolean(bool),
    PushInteger(i64),
    // PushFloat(f64),
    PushString(String),
    PushBlock(Vec<OwnedHighOp>),
    PushBuiltin,
}
