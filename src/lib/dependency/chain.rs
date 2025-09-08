use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Hash)] // Add PartialEq, Eq, Hash
pub struct DependencyChain<TDependency: Display> {
    pub chain: Vec<TDependency>,
    pub is_circular: bool,
    pub is_looped: bool,
}

impl<TDependency: Display> DependencyChain<TDependency> {
    pub fn new(chain: Vec<TDependency>, is_circular: bool, is_looped: bool) -> Self {
        DependencyChain { chain, is_circular, is_looped }
    }
}

