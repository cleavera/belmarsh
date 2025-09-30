use std::fmt::Display;
use std::hash::{Hash, Hasher};

use crate::module::Module;
use crate::repository::child::RepositoryChildPath;

pub mod chain;
pub mod cycle;
pub mod list;

#[derive(Debug, Clone)]
pub struct Dependency<TFrom: Display, TTo: Display> {
    pub from: TFrom,
    pub to: TTo,
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

    pub fn to_dot_format(&self) -> String {
        format!("  \"{}\" -> \"{}\";", self.from, self.to)
    }
}

impl<TFrom: Display, TTo: Display> Eq for Dependency<TFrom, TTo> {}

impl<TFrom: Display, TTo: Display> Hash for Dependency<TFrom, TTo> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from.to_string().hash(state);
        self.to.to_string().hash(state);
    }
}

impl<TFrom: Display, TTo: Display> Display for Dependency<TFrom, TTo> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} > {}", self.from, self.to)
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

impl<TTo: Display> Dependency<RepositoryChildPath, TTo> {
    pub fn is_from_module(&self, module_name: &str) -> bool {
        self.from
            .module()
            .map_or(false, |m| m.to_string() == module_name)
    }
}

impl Dependency<RepositoryChildPath, RepositoryChildPath> {
    pub fn is_internal(&self) -> bool {
        self.from.module().unwrap() == self.to.module().unwrap()
    }
}
