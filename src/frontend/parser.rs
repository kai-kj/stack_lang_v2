use {
    crate::{
        frontend::{
            lexer::{LLexError, LexError, Lexer},
            token::Token,
        },
        interpreter::high::operation::HighOp,
        location::{Located, LocatedExt, Location},
        session::{Block, BlockId, BuiltinId, Session, WordId},
    },
    std::collections::HashMap,
};

macro_rules! op {
    ($kind:ident $(($($value:expr),* $(,)?))?, $loc:expr) => {
        HighOp::$kind $(($($value),*))?.loc_copy($loc)
    };
}

macro_rules! error {
    ($kind:ident, $loc:expr) => {
        Err(ParseError::$kind.loc_copy($loc))
    };
}

pub fn parse(
    sesh: &mut Session,
    env: &mut Environment,
    lexer: &mut Lexer,
) -> Result<BlockId, LParseError> {
    let mut main = Block::new();
    while lexer.peek()?.val != Token::FileEnd {
        parse_op(sesh, env, lexer, &mut main)?;
    }
    Ok(sesh.blocks.push(main))
}

fn parse_op(
    sesh: &mut Session,
    env: &mut Environment,
    lexer: &mut Lexer,
    parent_block: &mut Block,
) -> Result<(), LParseError> {
    let curr_op = lexer.next(sesh)?;

    match curr_op.val {
        Token::BlockBegin => return error!(UnexpectedOpeningBracket, curr_op.loc),
        Token::BlockEnd => return error!(UnexpectedClosingBracket, curr_op.loc),
        Token::Define => parse_define(sesh, env, lexer)?,
        Token::Conditional => parse_conditional(sesh, env, lexer, parent_block, curr_op.loc)?,
        Token::Word(word_id) => parse_word(env, word_id, parent_block, curr_op.loc)?,
        Token::Boolean(v) => parent_block.push(op!(PushBoolean(v), curr_op.loc)),
        Token::Integer(v) => parent_block.push(op!(PushInteger(v), curr_op.loc)),
        Token::String(v) => parent_block.push(op!(PushString(v), curr_op.loc)),
        Token::FileEnd => return error!(UnexpectedEndOfFile, curr_op.loc),
    };

    Ok(())
}

fn parse_define(
    sesh: &mut Session,
    env: &mut Environment,
    lexer: &mut Lexer,
) -> Result<(), LParseError> {
    let next_op = lexer.next(sesh)?;
    let Token::Word(word_id) = next_op.val else {
        return error!(ExpectedWord, next_op.loc);
    };

    let mut function_env = env.child();
    let mut function_block = Block::new();
    parse_block(sesh, &mut function_env, lexer, &mut function_block)?;

    env.def(word_id, EnvironmentEntry::Block(sesh.blocks.push(function_block)));
    Ok(())
}

fn parse_conditional(
    sesh: &mut Session,
    env: &mut Environment,
    lexer: &mut Lexer,
    parent: &mut Block,
    parent_loc: Location,
) -> Result<(), LParseError> {
    let mut block_t = Block::new();
    parse_block(sesh, &mut env.child(), lexer, &mut block_t)?;

    let mut block_f = Block::new();
    parse_block(sesh, &mut env.child(), lexer, &mut block_f)?;

    parent.push(op!(Conditional(sesh.blocks.push(block_t), sesh.blocks.push(block_f)), parent_loc));
    Ok(())
}

fn parse_word(
    env: &mut Environment,
    word_id: WordId,
    parent_block: &mut Block,
    loc: Location,
) -> Result<(), LParseError> {
    match env.get(word_id).ok_or_else(|| ParseError::FunctionNotDefined.loc_copy(loc))? {
        EnvironmentEntry::Block(id) => parent_block.push(op!(CallBlock(id), loc)),
        EnvironmentEntry::Builtin(id) => parent_block.push(op!(CallBuiltin(id), loc)),
    }
    Ok(())
}

fn parse_block(
    sesh: &mut Session,
    env: &mut Environment,
    lexer: &mut Lexer,
    parent: &mut Block,
) -> Result<(), LParseError> {
    let start_op = lexer.next(sesh)?;
    if start_op.val != Token::BlockBegin {
        return error!(ExpectedOpeningBracket, start_op.loc);
    }
    loop {
        let next_op = *lexer.peek()?;
        match next_op.val {
            Token::BlockEnd => {
                lexer.next(sesh)?;
                return Ok(());
            }
            Token::FileEnd => return error!(UnexpectedEndOfFile, next_op.loc),
            _ => parse_op(sesh, env, lexer, parent)?,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Environment<'p> {
    parent: Option<&'p Environment<'p>>,
    binds: HashMap<WordId, EnvironmentEntry>,
}

impl<'p> Environment<'p> {
    pub fn new() -> Self {
        Self { parent: None, binds: HashMap::new() }
    }

    pub fn child(&'p self) -> Self {
        Self { parent: Some(self), binds: HashMap::new() }
    }

    pub fn def(&mut self, name: WordId, value: EnvironmentEntry) {
        self.binds.insert(name, value);
    }

    pub fn get(&self, name: WordId) -> Option<EnvironmentEntry> {
        if let Some(v) = self.binds.get(&name) {
            return Some(*v);
        }

        if let Some(parent) = self.parent {
            return parent.get(name);
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvironmentEntry {
    Block(BlockId),
    Builtin(BuiltinId),
}

pub type LParseError = Located<ParseError>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParseError {
    Lex(LexError),
    ExpectedOpeningBracket,
    ExpectedClosingBracket,
    UnexpectedOpeningBracket,
    UnexpectedClosingBracket,
    ExpectedWord,
    UnexpectedEndOfFile,
    FunctionNotDefined,
}

impl From<LLexError> for LParseError {
    fn from(error: LLexError) -> Self {
        ParseError::Lex(error.val).loc_copy(error.loc)
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::interpreter::{
            builtin::define_builtins, error::LInterpretError, high::operation::OwnedHighOp,
        },
    };

    macro_rules! block {
        ($($kind:ident $(($($args:tt)*))?),* $(,)?) => {
            Ok(vec![$(block!(@op $kind $(($($args)*))?)),*])
        };

        (@op CallBlock ($($kind:ident $(($($args:tt)*))?),* $(,)?)) => {
            OwnedHighOp::CallBlock(vec![$(block!(@op $kind $(($($args)*))?)),*])
        };

         (@op Conditional ($($kind_t:ident $(($($args_t:tt)*))?),* $(,)?; $($kind_f:ident $(($($args_f:tt)*))?),* $(,)?)) => {
            OwnedHighOp::Conditional(vec![$(block!(@op $kind_t $(($($args_t)*))?)),*], vec![$(block!(@op $kind_f $(($($args_f)*))?)),*])
        };

        (@op $kind:ident $(($value:expr))?) => {OwnedHighOp::$kind $(($value.into()))?};
    }

    fn parse_str_to_main(source_text: &str) -> Result<Vec<OwnedHighOp>, LInterpretError> {
        let mut sesh = Session::new();
        let mut env = Environment::new();
        define_builtins(&mut sesh, &mut env);

        let mut lexer = Lexer::new(&mut sesh, 0, source_text);
        let start = parse(&mut sesh, &mut env, &mut lexer)?;

        Ok(sesh.blocks.get(start).into_iter().map(|op| op.val.into_owned(&sesh)).collect::<_>())
    }

    #[test]
    fn test_boolean() {
        assert_eq!(parse_str_to_main("true"), block!(PushBoolean(true)));
    }

    #[test]
    fn test_integer() {
        assert_eq!(
            parse_str_to_main("-1 1 +1"),
            block!(PushInteger(-1), PushInteger(1), PushInteger(1))
        );
    }

    #[test]
    fn test_string() {
        assert_eq!(parse_str_to_main("\"Hello, world!\""), block!(PushString("Hello, world!")));
    }

    #[test]
    fn test_def_and_get() {
        assert_eq!(parse_str_to_main("#def foo [1] foo"), block!(CallBlock(PushInteger(1))));
    }

    #[test]
    fn test_conditional() {
        assert_eq!(
            parse_str_to_main("true #cond [1] [2]"),
            block!(
                PushBoolean(true),
                Conditional(PushInteger(1); PushInteger(2))
            )
        );
    }
}
