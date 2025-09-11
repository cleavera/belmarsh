use std::{
    hash::{Hash, Hasher},
    collections::{HashMap, HashSet},
    fmt::Display,
};

use rayon::prelude::*;

#[derive(Clone, Debug)]
pub struct DependencyChain<TDependency: Display>(Vec<TDependency>);

impl<TDependency: Clone + Display> DependencyChain<TDependency> {
    fn normalize(&self) -> Vec<TDependency> {
        if self.0.is_empty() {
            return vec![];
        }

        let min_index = self
            .0
            .iter()
            .enumerate()
            .min_by_key(|&(_, item)| item.to_string())
            .map(|(i, _)| i)
            .unwrap_or(0);

        let mut normalized = self.0.clone();
        normalized.rotate_left(min_index);
        normalized
    }
}

impl<TDependency: Display + Clone> Hash for DependencyChain<TDependency> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.normalize().iter().for_each(|item| item.to_string().hash(state));
    }
}

impl<TDependency: Display + Clone> PartialEq for DependencyChain<TDependency> {
    fn eq(&self, other: &Self) -> bool {
        let self_normalized = self.normalize();
        let other_normalized = other.normalize();

        if self_normalized.len() != other_normalized.len() {
            return false;
        }

        self_normalized.iter().zip(other_normalized.iter()).all(|(a, b)| a.to_string() == b.to_string())
    }
}

impl<TDependency: Display> Display for DependencyChain<TDependency> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().map(|d| d.to_string()).collect::<Vec<String>>().join(" > "))
    }
}

impl<TDependency: Display + Clone> Eq for DependencyChain<TDependency> {}

impl<TDependency: Clone + Display> DependencyChain<TDependency> {
    pub fn is_circular(&self) -> bool {
        self.0.len() > 2
            && self.0.first().unwrap().to_string() == self.0.last().unwrap().to_string()
    }

    pub fn has_loop(&self) -> bool {
        let unique_items: HashSet<_> = self.0.iter().map(|x| x.to_string()).collect();
        unique_items.len() < self.0.len()
    }

    pub fn extend(&self, next_value: TDependency) -> DependencyChain<TDependency> {
        let mut new_chain = self.0.clone();
    
        new_chain.push(next_value);

        DependencyChain(new_chain)
    }
}

pub struct DependencyChainListBuilder<TDependency: Display> {
    grouped_dependencies: HashMap<TDependency, Vec<TDependency>>,
}

impl<TDependency: Clone + Display + Eq + Hash + Send + Sync> DependencyChainListBuilder<TDependency> {
    pub fn build<TDep: Clone + Display + Eq + Hash + Send + Sync>(
        grouped_dependencies: HashMap<TDep, Vec<TDep>>,
    ) -> HashSet<DependencyChain<TDep>> {
        let builder = DependencyChainListBuilder {
            grouped_dependencies,
        };

        builder.grouped_dependencies.par_iter().flat_map(|(key, dependencies)| {
            let chain = DependencyChain(vec![(*key).clone()]);

            builder.dfs(chain, dependencies)
        }).collect()
    }

    fn dfs(
        &self,
        chain: DependencyChain<TDependency>,
        dependencies: &Vec<TDependency>,
    ) -> HashSet<DependencyChain<TDependency>> {
        if dependencies.len() == 0 || chain.is_circular() || chain.has_loop() {
            let mut hs = HashSet::new();

            hs.insert(chain);

            return hs;
        }

        dependencies.iter().flat_map(|dependency| {
            self.dfs(
                chain.extend((*dependency).clone()),
                self.grouped_dependencies.get(dependency).unwrap_or(&vec![]),
            )
        }).collect::<HashSet<DependencyChain<TDependency>>>()
    }
}
