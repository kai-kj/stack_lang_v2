use {
    crate::{
        interpreter::{error::InterpretError, high::operation::LHighOp, stack::Stack},
        util::{Store, StoreId},
    },
    std::{collections::HashMap, rc::Rc},
};

pub struct Session {
    pub words: WordTable,
    pub strings: Store<String>,
    pub blocks: Store<Block>,
    pub builtins: Store<Builtin>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            words: WordTable::new(),
            strings: Store::new(),
            blocks: Store::new(),
            builtins: Store::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordTable {
    id_to_name: Vec<Rc<str>>,
    name_to_id: HashMap<Rc<str>, WordId>,
}

impl WordTable {
    pub fn new() -> Self {
        Self { id_to_name: Vec::new(), name_to_id: HashMap::new() }
    }

    pub fn push(&mut self, name: &str) -> WordId {
        if let Some(&id) = self.name_to_id.get(name) {
            return id;
        }

        let id = WordId(self.id_to_name.len());
        let name: Rc<str> = Rc::from(name);

        self.id_to_name.push(Rc::clone(&name));
        self.name_to_id.insert(name, id);

        id
    }

    pub fn get(&self, id: WordId) -> &str {
        &self.id_to_name[id.0]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WordId(usize);

pub type StringId = StoreId<String>;

pub type BlockId = StoreId<Block>;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    operations: Vec<LHighOp>,
}

impl Block {
    pub fn new() -> Self {
        Self { operations: Vec::new() }
    }

    pub fn push(&mut self, op: LHighOp) {
        self.operations.push(op);
    }
}

impl<'i> IntoIterator for &'i Block {
    type Item = &'i LHighOp;
    type IntoIter = std::slice::Iter<'i, LHighOp>;

    fn into_iter(self) -> Self::IntoIter {
        self.operations.iter()
    }
}

pub type BuiltinId = StoreId<Builtin>;

#[derive(Debug, Clone, PartialEq)]
pub struct Builtin {
    function: fn(&mut Stack) -> Result<(), InterpretError>,
}

impl Builtin {
    pub fn new(function: fn(&mut Stack) -> Result<(), InterpretError>) -> Self {
        Self { function }
    }

    pub fn call(&self, stack: &mut Stack) -> Result<(), InterpretError> {
        (self.function)(stack)
    }
}
