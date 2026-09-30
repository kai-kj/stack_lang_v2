use std::{hash::Hash, marker::PhantomData};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct StoreId<V>(usize, PhantomData<V>);

impl<V> Copy for StoreId<V> {}

impl<V> Clone for StoreId<V> {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Store<V> {
    strings: Vec<V>,
}

impl<V> Store<V> {
    pub fn new() -> Self {
        Self { strings: Vec::new() }
    }

    pub fn push(&mut self, value: V) -> StoreId<V> {
        let id = StoreId(self.strings.len(), PhantomData);
        self.strings.push(value);
        id
    }

    pub fn get(&self, id: StoreId<V>) -> &V {
        &self.strings[id.0]
    }
}
