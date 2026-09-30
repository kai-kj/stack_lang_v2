use crate::{
    frontend::parser::{LParseError, ParseError},
    location::{Located, LocatedExt},
};

pub type LInterpretError = Located<InterpretError>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterpretError {
    Parse(ParseError),
    StackIsEmpty,
    ValueIsNotCallable,
    UnexpectedType,
}

impl From<LParseError> for LInterpretError {
    fn from(error: LParseError) -> Self {
        InterpretError::Parse(error.val).loc_copy(error.loc)
    }
}
