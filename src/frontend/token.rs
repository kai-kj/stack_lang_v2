use crate::{
    location::Located,
    session::{Session, StringId, WordId},
};

pub type LToken = Located<Token>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    BlockBegin,
    BlockEnd,
    Conditional,
    Define,
    Word(WordId),
    Boolean(bool),
    Integer(i64),
    // Float(f64),
    String(StringId),
    FileEnd,
}

impl Token {
    pub fn into_owned(self, sesh: &Session) -> OwnedToken {
        match self {
            Token::BlockBegin => OwnedToken::BlockBegin,
            Token::BlockEnd => OwnedToken::BlockEnd,
            Token::Conditional => OwnedToken::Conditional,
            Token::Define => OwnedToken::Define,
            Token::Word(id) => OwnedToken::Word(sesh.words.get(id).to_string()),
            Token::Boolean(v) => OwnedToken::Boolean(v),
            Token::Integer(v) => OwnedToken::Integer(v),
            Token::String(id) => OwnedToken::String(sesh.strings.get(id).to_string()),
            Token::FileEnd => OwnedToken::FileEnd,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OwnedToken {
    BlockBegin,
    BlockEnd,
    Quote,
    Conditional,
    Define,
    Word(String),
    Boolean(bool),
    Integer(i64),
    // Float(f64),
    String(String),
    FileEnd,
}
