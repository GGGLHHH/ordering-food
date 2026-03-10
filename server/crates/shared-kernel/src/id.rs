use std::{fmt::Debug, hash::Hash};

pub trait Identifier: Clone + Debug + Eq + Hash + Send + Sync + 'static {
    fn as_str(&self) -> &str;
}

pub trait AggregateId: Identifier {}
