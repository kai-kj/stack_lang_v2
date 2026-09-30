use crate::{
    frontend::parser::{Environment, EnvironmentEntry},
    interpreter::{error::InterpretError, value::Value},
    session::{Builtin, Session},
};

macro_rules! pop {
    ($stack:expr, $type:ident) => {{
        let Value::$type(value) = $stack.pop()? else {
            return Err(InterpretError::UnexpectedType);
        };
        value
    }};
}

macro_rules! push {
    ($stack:expr, $type:ident, $value:expr) => {
        $stack.push(Value::$type($value))
    };
}

pub fn define_builtins(sesh: &mut Session, env: &mut Environment) {
    define_builtin(
        sesh,
        env,
        "+",
        Builtin::new(|stack| {
            let a = pop!(stack, Integer);
            let b = pop!(stack, Integer);
            push!(stack, Integer, a + b);
            Ok(())
        }),
    )
}

fn define_builtin(sesh: &mut Session, env: &mut Environment, name: &str, builtin: Builtin) {
    let word_id = sesh.words.push(name);
    let builtin_id = sesh.builtins.push(builtin);
    env.def(word_id, EnvironmentEntry::Builtin(builtin_id));
}
