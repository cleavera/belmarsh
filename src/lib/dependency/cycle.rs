use std::collections::{HashMap, HashSet};

use super::chain::DependencyChain;

enum NodeState {
    Visiting,
    Visited,
}

pub struct CycleDetector {
    grouped_dependencies: HashMap<String, Vec<String>>,
    states: HashMap<String, NodeState>,
    cycles: HashSet<DependencyChain>,
}

impl CycleDetector {
    pub fn new(grouped_dependencies: HashMap<String, Vec<String>>) -> Self {
        CycleDetector {
            grouped_dependencies,
            states: HashMap::new(),
            cycles: HashSet::new(),
        }
    }

    pub fn find_cycles(mut self) -> HashSet<DependencyChain> {
        let keys: Vec<String> = self.grouped_dependencies.keys().cloned().collect();
        for node in keys {
            if !self.states.contains_key(&node) {
                self.dfs(&node, &mut vec![]);
            }
        }
        self.cycles
    }

    fn dfs(&mut self, current_node: &String, path: &mut Vec<String>) {
        self.states
            .insert(current_node.clone(), NodeState::Visiting);
        path.push(current_node.clone());

        let dependencies_to_check =
            self.grouped_dependencies.get(current_node).cloned().unwrap_or_default();

        for dependency in dependencies_to_check {
            match self.states.get(&dependency) {
                Some(NodeState::Visiting) => {
                    if let Some(cycle_start_index) = path.iter().position(|p| p == &dependency) {
                        let mut cycle_chain_vec = path[cycle_start_index..].to_vec();
                        cycle_chain_vec.push(dependency.clone());
                        self.cycles.insert(DependencyChain(cycle_chain_vec));
                    }
                }

                Some(NodeState::Visited) => (),
                None => {
                    self.dfs(&dependency, path);
                }
            }
        }

        path.pop();
        self.states
            .insert(current_node.clone(), NodeState::Visited);
    }
}
