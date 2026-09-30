use {
    crate::{
        interpreter::{
            error::LInterpretError,
            high::operation::{HighOp, LHighOp},
            stack::Stack,
            value::Value,
        },
        location::LocatedResultExt,
        session::{BlockId, Session},
    },
    std::rc::Rc,
};

pub struct HighLevelInterpreter {
    stack: Stack,
}

macro_rules! error {
    ($kind:ident, $loc:expr) => {
        Err(InterpretError::$kind.loc_copy($loc))
    };
}

impl HighLevelInterpreter {
    pub fn new() -> Self {
        Self { stack: Stack::new() }
    }

    pub fn interpret(&mut self, sesh: &Session, start: BlockId) -> Result<(), LInterpretError> {
        self.interpret_block(sesh, start)
    }

    fn interpret_block(&mut self, sesh: &Session, start: BlockId) -> Result<(), LInterpretError> {
        let block = sesh.blocks.get(start);

        for op in block {
            self.interpret_op(sesh, op)?;
        }

        Ok(())
    }

    fn interpret_op(&mut self, sesh: &Session, op: &LHighOp) -> Result<(), LInterpretError> {
        match op.val {
            HighOp::PushBoolean(value) => self.stack.push(Value::Boolean(value)),
            HighOp::PushInteger(value) => self.stack.push(Value::Integer(value)),
            HighOp::PushString(value) => {
                let value = sesh.strings.get(value);
                self.stack.push(Value::String(Rc::from(value.as_str())))
            }
            HighOp::CallBlock(id) => self.interpret_block(sesh, id)?,
            HighOp::CallBuiltin(id) => {
                sesh.builtins.get(id).call(&mut self.stack).err_loc_copy(op.loc)?
            }
            HighOp::Conditional(id_t, id_f) => {
                let cond = self.stack.pop().err_loc_copy(op.loc)?;
                if cond.is_truthy() {
                    self.interpret_block(sesh, id_t)?;
                } else {
                    self.interpret_block(sesh, id_f)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            frontend::{
                lexer::Lexer,
                parser::{Environment, parse},
            },
            interpreter::builtin::define_builtins,
        },
    };

    macro_rules! stack {
        ($($kind:ident $(($value:expr))?),* $(,)?) => {{
            let mut stack = Stack::new();
            $(stack.push(stack!(@val $kind $(($value))?));)*
            Ok(stack)
        }};

        (@val $kind:ident $(($value:expr))?) => {Value::$kind $(($value.into()))?};
    }

    fn interpret_str_to_stack(source_text: &str) -> Result<Stack, LInterpretError> {
        let mut sesh = Session::new();
        let mut env = Environment::new();
        define_builtins(&mut sesh, &mut env);

        let mut lexer = Lexer::new(&mut sesh, 0, source_text);
        let start = parse(&mut sesh, &mut env, &mut lexer)?;

        let mut interpreter = HighLevelInterpreter::new();
        interpreter.interpret(&mut sesh, start)?;

        Ok(interpreter.stack)
    }

    #[test]
    fn test_builtin() {
        assert_eq!(interpret_str_to_stack("1 2 +"), stack!(Integer(3)))
    }

    #[test]
    fn test_user_defined() {
        assert_eq!(interpret_str_to_stack("#def inc [1 +] 2 inc"), stack!(Integer(3)))
    }

    #[test]
    fn test_conditional_a() {
        assert_eq!(interpret_str_to_stack("true #cond [1] [2]"), stack!(Integer(1)))
    }

    #[test]
    fn test_conditional_b() {
        assert_eq!(interpret_str_to_stack("false #cond [1] [2]"), stack!(Integer(2)))
    }
}
