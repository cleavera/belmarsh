use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    hash::{Hash, Hasher},
};

use rayon::prelude::*;

#[derive(Clone, Debug, Eq)]
pub struct DependencyChain(pub Vec<String>);

impl DependencyChain {
    fn normalize(&self) -> Vec<String> {
        if self.0.is_empty() {
            return vec![];
        }

        let min_index = self
            .0
            .iter()
            .enumerate()
            .min_by_key(|&(_, item)| item)
            .map(|(i, _)| i)
            .unwrap_or(0);

        let mut normalized = self.0.clone();
        normalized.rotate_left(min_index);
        normalized
    }

    pub fn is_circular(&self) -> bool {
        self.0.len() > 2 && self.0.first().unwrap() == self.0.last().unwrap()
    }

    pub fn has_loop(&self) -> bool {
        let unique_items: HashSet<_> = self.0.iter().collect();
        unique_items.len() < self.0.len()
    }

    pub fn extend(&self, next_value: String) -> DependencyChain {
        let mut new_chain = self.0.clone();
        new_chain.push(next_value);
        DependencyChain(new_chain)
    }
}

impl Hash for DependencyChain {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.normalize().iter().for_each(|item| item.hash(state));
    }
}

impl PartialEq for DependencyChain {
    fn eq(&self, other: &Self) -> bool {
        self.normalize() == other.normalize()
    }
}

impl Display for DependencyChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join(" > "))
    }
}

pub struct DependencyChainListBuilder {
    grouped_dependencies: HashMap<String, Vec<String>>,
}

impl DependencyChainListBuilder {
    pub fn build(
        grouped_dependencies: HashMap<String, Vec<String>>,
    ) -> HashSet<DependencyChain> {
        let builder = DependencyChainListBuilder {
            grouped_dependencies,
        };

        builder
            .grouped_dependencies
            .par_iter()
            .flat_map(|(key, dependencies)| {
                let chain = DependencyChain(vec![key.clone()]);
                builder.dfs(chain, dependencies)
            })
            .collect()
    }

    fn dfs(
        &self,
        chain: DependencyChain,
        dependencies: &Vec<String>,
    ) -> HashSet<DependencyChain> {
        if dependencies.is_empty() || chain.is_circular() || chain.has_loop() {
            let mut hs = HashSet::new();

            hs.insert(chain);

            return hs;
        }

        dependencies
            .iter()
            .flat_map(|dependency| {
                self.dfs(
                    chain.extend(dependency.clone()),
                    self.grouped_dependencies
                        .get(dependency)
                        .unwrap_or(&vec![]),
                )
            })
            .collect::<HashSet<DependencyChain>>()
    }
}
