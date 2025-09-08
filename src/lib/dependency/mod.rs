use std::fmt::Display;
use std::hash::{Hash, Hasher};

use crate::module::Module;

pub mod list;

#[derive(Debug)]
pub struct Dependency<TFrom: Display, TTo: Display> {
    pub(in crate::dependency) from: TFrom,
    pub(in crate::dependency) to: TTo,
}

impl<TFrom: Display, TTo: Display> PartialEq for Dependency<TFrom, TTo> {
    fn eq(&self, other: &Self) -> bool {
        self.from.to_string() == other.from.to_string()
            && self.to.to_string() == other.to.to_string()
    }
}

impl<TFrom: Display, TTo: Display> Dependency<TFrom, TTo> {
    pub fn create(from: TFrom, to: TTo) -> Dependency<TFrom, TTo> {
        Dependency { from, to }
    }
}

impl<TFrom: Display, TTo: Display> Eq for Dependency<TFrom, TTo> {}

impl<TFrom: Display, TTo: Display> Hash for Dependency<TFrom, TTo> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from.to_string().hash(state);
        self.to.to_string().hash(state);
    }
}

impl Dependency<Module, Module> {
    pub fn is_internal(&self) -> bool {
        self.from.to_string() == self.to.to_string()
    }
}

impl Dependency<&Module, &Module> {
    pub fn is_internal(&self) -> bool {
        self.from.to_string() == self.to.to_string()
    }
}
