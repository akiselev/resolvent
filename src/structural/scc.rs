//! Directed graph and deterministic strongly-connected-component analysis used by the
//! structural compiler. The implementation is iterative Tarjan: deep generated equation
//! systems cannot overflow the Rust call stack.

use serde::{Deserialize, Serialize};
use std::fmt;

const UNVISITED: usize = usize::MAX;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Digraph {
    adj: Vec<Vec<usize>>,
}

impl Digraph {
    pub fn new(n_nodes: usize) -> Self {
        Self {
            adj: vec![Vec::new(); n_nodes],
        }
    }

    pub fn with_edges(
        n_nodes: usize,
        edges: impl IntoIterator<Item = (usize, usize)>,
    ) -> Result<Self, GraphError> {
        let mut graph = Self::new(n_nodes);
        for (from, to) in edges {
            graph.add_edge(from, to)?;
        }
        graph.normalize();
        Ok(graph)
    }

    pub fn add_edge(&mut self, from: usize, to: usize) -> Result<(), GraphError> {
        let n_nodes = self.adj.len();
        if from >= n_nodes {
            return Err(GraphError::NodeOutOfBounds {
                node: from,
                n_nodes,
            });
        }
        if to >= n_nodes {
            return Err(GraphError::NodeOutOfBounds { node: to, n_nodes });
        }
        self.adj[from].push(to);
        Ok(())
    }

    pub fn n_nodes(&self) -> usize {
        self.adj.len()
    }

    pub fn successors(&self, node: usize) -> &[usize] {
        self.adj.get(node).map_or(&[], Vec::as_slice)
    }

    pub fn normalize(&mut self) {
        for successors in &mut self.adj {
            successors.sort_unstable();
            successors.dedup();
        }
    }
}

/// SCCs are emitted in reverse topological order of the condensation graph (sinks first),
/// which is directly useful as block-lower-triangular solve order for dependency edges of
/// the form `consumer -> producer`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sccs {
    components: Vec<Vec<usize>>,
    component_of: Vec<usize>,
}

impl Sccs {
    pub fn components(&self) -> &[Vec<usize>] {
        &self.components
    }

    pub fn component_of(&self, node: usize) -> Option<usize> {
        self.component_of.get(node).copied()
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

pub fn tarjan_scc(graph: &Digraph) -> Sccs {
    let n = graph.n_nodes();
    let mut next_index = 0usize;
    let mut index = vec![UNVISITED; n];
    let mut lowlink = vec![0usize; n];
    let mut on_stack = vec![false; n];
    let mut scc_stack = Vec::new();
    let mut components = Vec::new();
    let mut component_of = vec![0usize; n];
    let mut work: Vec<(usize, usize)> = Vec::new();

    for start in 0..n {
        if index[start] != UNVISITED {
            continue;
        }
        work.push((start, 0));

        while !work.is_empty() {
            let top = work.len() - 1;
            let (node, cursor) = work[top];
            if cursor == 0 && index[node] == UNVISITED {
                index[node] = next_index;
                lowlink[node] = next_index;
                next_index += 1;
                scc_stack.push(node);
                on_stack[node] = true;
            }

            let successors = graph.successors(node);
            if cursor < successors.len() {
                let successor = successors[cursor];
                work[top].1 += 1;
                if index[successor] == UNVISITED {
                    work.push((successor, 0));
                } else if on_stack[successor] {
                    lowlink[node] = lowlink[node].min(index[successor]);
                }
                continue;
            }

            if lowlink[node] == index[node] {
                let mut component = Vec::new();
                while let Some(member) = scc_stack.pop() {
                    on_stack[member] = false;
                    component_of[member] = components.len();
                    component.push(member);
                    if member == node {
                        break;
                    }
                }
                component.sort_unstable();
                components.push(component);
            }

            work.pop();
            if let Some(&(parent, _)) = work.last() {
                lowlink[parent] = lowlink[parent].min(lowlink[node]);
            }
        }
    }

    Sccs {
        components,
        component_of,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphError {
    NodeOutOfBounds { node: usize, n_nodes: usize },
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeOutOfBounds { node, n_nodes } => write!(
                f,
                "edge references node {node} but graph has only {n_nodes} node(s)"
            ),
        }
    }
}

impl std::error::Error for GraphError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn components_are_reverse_topological() {
        let graph = Digraph::with_edges(4, [(0, 1), (1, 0), (1, 2), (2, 3), (3, 2)]).unwrap();
        let sccs = tarjan_scc(&graph);
        assert_eq!(sccs.components(), &[vec![2, 3], vec![0, 1]]);
    }

    #[test]
    fn deep_chain_is_iterative_and_deterministic() {
        let n = 10_000;
        let graph = Digraph::with_edges(n, (0..n - 1).map(|i| (i, i + 1))).unwrap();
        let sccs = tarjan_scc(&graph);
        assert_eq!(sccs.len(), n);
        assert_eq!(sccs.components()[0], vec![n - 1]);
        assert_eq!(sccs.components()[n - 1], vec![0]);
    }
}
