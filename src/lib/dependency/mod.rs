use std::fmt::Display;

use crate::module::Module;

pub mod list;

#[derive(Debug)]
pub struct Dependency<TFrom: Display, TTo: Display> {
    from: TFrom,
    to: TTo,
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
