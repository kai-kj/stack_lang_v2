use crate::{
    location::Located,
    session::{BlockId, BuiltinId, Session, StringId},
};

pub type LHighOp = Located<HighOp>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HighOp {
    PushBoolean(bool),
    PushInteger(i64),
    PushString(StringId),
    CallBlock(BlockId),
    CallBuiltin(BuiltinId),
    Conditional(BlockId, BlockId),
}

impl HighOp {
    pub fn into_owned(self, sesh: &Session) -> OwnedHighOp {
        match self {
            HighOp::PushBoolean(v) => OwnedHighOp::PushBoolean(v),
            HighOp::PushInteger(v) => OwnedHighOp::PushInteger(v),
            HighOp::PushString(id) => OwnedHighOp::PushString(sesh.strings.get(id).to_string()),
            HighOp::CallBlock(block) => OwnedHighOp::CallBlock(
                sesh.blocks.get(block).into_iter().map(|op| op.val.into_owned(sesh)).collect(),
            ),
            HighOp::CallBuiltin(_) => OwnedHighOp::CallBuiltin,
            HighOp::Conditional(block_t, block_f) => OwnedHighOp::Conditional(
                sesh.blocks.get(block_t).into_iter().map(|op| op.val.into_owned(sesh)).collect(),
                sesh.blocks.get(block_f).into_iter().map(|op| op.val.into_owned(sesh)).collect(),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OwnedHighOp {
    PushBoolean(bool),
    PushInteger(i64),
    PushString(String),
    CallBlock(Vec<OwnedHighOp>),
    CallBuiltin,
    Conditional(Vec<OwnedHighOp>, Vec<OwnedHighOp>),
}
