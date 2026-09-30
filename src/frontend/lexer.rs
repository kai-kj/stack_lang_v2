use crate::{
    frontend::token::{LToken, Token},
    location::{Located, LocatedExt},
    session::Session,
};

pub struct Lexer<'s> {
    source_index: usize,
    source_text: &'s str,
    next_pos: usize,
    next_event: Result<LToken, LLexError>,
}

macro_rules! token {
    ($self:expr, $sesh:expr, Word($value:expr), $start:expr, $end:expr) => {{
        let string_id = $sesh.words.push($value);
        Ok(Token::Word(string_id).loc_at($self.source_index, $start, $end))
    }};

    ($self:expr, $sesh:expr, Str($value:expr), $start:expr, $end:expr) => {{
        let string_id = $sesh.strings.push($value);
        Ok(Token::String(string_id).loc_at($self.source_index, $start, $end))
    }};

    ($self:expr, $sesh:expr, $kind:ident $(($value:expr))?, $start:expr, $end:expr) => {
        Ok(Token::$kind $(($value))?.loc_at($self.source_index, $start, $end))
    };
}

macro_rules! event_adv {
    ($self:expr, $sesh:expr, $kind:ident $(($value:expr))?, $start:expr, $end:expr) => {{
        $self.advance();
        token!($self, $sesh, $kind $(($value))?, $start, $end)
    }};
}

macro_rules! error {
    ($self:expr, $kind:ident, $start:expr, $end:expr) => {
        Err(LexError::$kind.loc_at($self.source_index, $start, $end))
    };
}

impl<'s> Lexer<'s> {
    pub fn new(sesh: &mut Session, source_index: usize, source_text: &'s str) -> Self {
        let mut lexer = Self {
            source_index,
            source_text,
            next_pos: 0,
            next_event: Ok(Token::FileEnd.loc_at(source_index, 0, 0)),
        };
        lexer.next_event = lexer.lex(sesh);
        lexer
    }

    pub fn peek(&self) -> Result<&LToken, LLexError> {
        self.next_event.as_ref().map_err(|err| *err)
    }

    pub fn next(&mut self, sesh: &mut Session) -> Result<LToken, LLexError> {
        let next = self.lex(sesh);
        std::mem::replace(&mut self.next_event, next)
    }

    fn lex(&mut self, sesh: &mut Session) -> Result<LToken, LLexError> {
        self.advance_while(|_, c| c.is_whitespace());
        let start_pos = self.next_pos;

        match self.curr_char() {
            Some('[') => event_adv!(self, sesh, BlockBegin, start_pos, start_pos + 1),
            Some(']') => event_adv!(self, sesh, BlockEnd, start_pos, start_pos + 1),
            Some('"') => self.lex_string(sesh, start_pos),
            Some(_) => self.lex_other(sesh, start_pos),
            None => {
                event_adv!(self, sesh, FileEnd, self.source_text.len(), self.source_text.len() + 1)
            }
        }
    }

    fn lex_string(&mut self, sesh: &mut Session, start_pos: usize) -> Result<LToken, LLexError> {
        let mut res = String::new();
        self.advance();
        loop {
            match self.curr_char() {
                Some('"') => return event_adv!(self, sesh, Str(res), start_pos, self.next_pos + 1),
                Some('\\') => {
                    let sequence_pos = self.next_pos;
                    self.advance();
                    res.push(match self.curr_char() {
                        Some('n') => '\n',
                        Some('r') => '\r',
                        Some('t') => '\t',
                        Some('0') => '\0',
                        Some('\\') => '\\',
                        Some('"') => '"',
                        _ => {
                            return error!(
                                self,
                                InvalidEscapeSequence,
                                sequence_pos,
                                self.next_pos + 1
                            );
                        }
                    });
                }
                Some(c) => res.push(c),
                None => return error!(self, UnterminatedString, start_pos, self.next_pos),
            }
            self.advance();
        }
    }

    fn lex_other(&mut self, sesh: &mut Session, start_pos: usize) -> Result<LToken, LLexError> {
        self.advance_while(|_, c| !c.is_whitespace() && !matches!(c, '[' | ']' | '"'));
        let chars = &self.source_text[start_pos..self.next_pos];

        if let Ok(number) = chars.parse::<i64>() {
            return token!(self, sesh, Integer(number), start_pos, self.next_pos);
        }

        // if let Ok(number) = chars.parse::<f64>() {
        //     return token!(self, sesh, Float(number), start_pos, self.next_pos);
        // }

        match chars {
            "#cond" => token!(self, sesh, Conditional, start_pos, self.next_pos),
            "#def" => token!(self, sesh, Define, start_pos, self.next_pos),
            "true" => token!(self, sesh, Boolean(true), start_pos, self.next_pos),
            "false" => token!(self, sesh, Boolean(false), start_pos, self.next_pos),
            c if c.starts_with('#') => error!(self, InvalidDirective, start_pos, self.next_pos),
            _ => token!(self, sesh, Word(chars), start_pos, self.next_pos),
        }
    }

    fn curr_char(&self) -> Option<char> {
        self.source_text[self.next_pos..].chars().next()
    }

    fn prev_char(&self) -> Option<char> {
        self.source_text[..self.next_pos].chars().next_back()
    }

    fn advance(&mut self) {
        if let Some(c) = self.curr_char() {
            self.next_pos += c.len_utf8();
        }
    }

    fn advance_while(&mut self, condition: fn(Option<char>, char) -> bool) {
        while let Some(curr) = self.curr_char() {
            if !condition(self.prev_char(), curr) {
                break;
            }
            self.advance();
        }
    }
}

pub type LLexError = Located<LexError>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LexError {
    InvalidEscapeSequence,
    UnterminatedString,
    InvalidDirective,
}

#[cfg(test)]
mod tests {
    use {super::*, crate::frontend::token::OwnedToken};

    macro_rules! tokens {
        ($($kind:ident $(($value:expr))?),* $(,)?) => {
            Ok(vec![$(tokens!(@tok $kind $(($value))?)),*])
        };

        // (@tok $sesh:expr, Word($value:expr)) => {{
        //     let string_id = $sesh.words.push($value);
        //     Token::Word(string_id)
        // }};
        //
        // (@tok $sesh:expr, String($value:expr)) => {{
        //     let string_id = $sesh.strings.push($value.into());
        //     Token::String(string_id)
        // }};

        (@tok $kind:ident $(($value:expr))?) => {OwnedToken::$kind $(($value.into()))?};
    }

    macro_rules! error {
        ($kind:ident, $start:expr, $end:expr) => {
            Err(LexError::$kind.loc_at(0, $start, $end))
        };
    }

    fn lex_str_to_owned_token_vec(source_text: &str) -> Result<Vec<OwnedToken>, LLexError> {
        let mut sesh = Session::new();
        let mut lexer = Lexer::new(&mut sesh, 0, source_text);
        let mut events = vec![];
        loop {
            match lexer.next(&mut sesh) {
                Err(e) => return Err(e),
                Ok(token) if token.val == Token::FileEnd => break,
                Ok(token) => events.push(token.val.into_owned(&mut sesh)),
            }
        }
        Ok(events)
    }

    #[test]
    fn test_empty() {
        assert_eq!(lex_str_to_owned_token_vec(""), tokens!());
    }

    #[test]
    fn test_block() {
        assert_eq!(
            lex_str_to_owned_token_vec(r#"[][[]]"#),
            tokens!(BlockBegin, BlockEnd, BlockBegin, BlockBegin, BlockEnd, BlockEnd)
        );
    }

    #[test]
    fn test_define() {
        assert_eq!(
            lex_str_to_owned_token_vec(r#"#def x 42"#),
            tokens!(Define, Word("x"), Integer(42))
        );
    }

    #[test]
    fn test_word() {
        assert_eq!(
            lex_str_to_owned_token_vec(r#"foo bar +"#),
            tokens!(Word("foo"), Word("bar"), Word("+"))
        );
    }

    #[test]
    fn test_int() {
        assert_eq!(
            lex_str_to_owned_token_vec(r#"-42 42 +42"#),
            tokens!(Integer(-42), Integer(42), Integer(42))
        );
    }

    #[test]
    fn test_int_zero() {
        assert_eq!(
            lex_str_to_owned_token_vec(r#"-0 0 +0"#),
            tokens!(Integer(0), Integer(0), Integer(0))
        );
    }

    // #[test]
    // fn test_float() {
    //     let mut sesh = Session::new();
    //     assert_eq!(
    //         lex_str_to_event_vec(&mut sesh, r#"-42.0 42.0 +42.0"#),
    //         tokens!(sesh, Float(-42.0), Float(42.0), Float(42.0))
    //     );
    // }
    //
    // #[test]
    // fn test_float_zero() {
    //     let mut sesh = Session::new();
    //     assert_eq!(
    //         lex_str_to_event_vec(&mut sesh, r#"-0.0 0.0 +0.0"#),
    //         tokens!(sesh, Float(-0.0), Float(0.0), Float(0.0))
    //     );
    // }

    #[test]
    fn test_string() {
        assert_eq!(
            lex_str_to_owned_token_vec(r#""Hello, world!""#),
            tokens!(String("Hello, world!"))
        );
    }

    #[test]
    fn test_string_escaped_a() {
        assert_eq!(
            lex_str_to_owned_token_vec(r#""Hello, \"world\"!""#),
            tokens!(String("Hello, \"world\"!"))
        );
    }

    #[test]
    fn test_string_escaped_b() {
        assert_eq!(
            lex_str_to_owned_token_vec(r#""Hello,\tworld!""#),
            tokens!(String("Hello,	world!"))
        );
    }

    #[test]
    fn test_parse_string_escaped_c() {
        assert_eq!(
            lex_str_to_owned_token_vec(r#""Hello, \\world!""#),
            tokens!(String("Hello, \\world!"))
        );
    }

    #[test]
    fn test_no_whitespace() {
        assert_eq!(
            lex_str_to_owned_token_vec(r#"[foo[bar baz]+]"#),
            tokens!(
                BlockBegin,
                Word("foo"),
                BlockBegin,
                Word("bar"),
                Word("baz"),
                BlockEnd,
                Word("+"),
                BlockEnd
            )
        );
    }

    #[test]
    fn test_unterminated_string() {
        assert_eq!(
            lex_str_to_owned_token_vec(r#""Hello, world!"#),
            error!(UnterminatedString, 0, 14)
        );
    }

    #[test]
    fn test_unexpected_escape_sequence() {
        assert_eq!(
            lex_str_to_owned_token_vec(r#""Hello, \world!"#),
            error!(InvalidEscapeSequence, 8, 10)
        );
    }

    #[test]
    fn test_invalid_directive() {
        assert_eq!(lex_str_to_owned_token_vec(r#"#foo"#), error!(InvalidDirective, 0, 4));
    }
}
