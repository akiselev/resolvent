//! Structural equation-system compiler passes.
//!
//! These algorithms operate on [`crate::system::System`] semantics rather than owning a
//! second equation language. They are the long-term home for the matching/SCC/BLT/tearing
//! capabilities currently implemented by Plexus.

use std::collections::{BTreeMap, VecDeque};

use crate::system::{System, VariableId};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StructuralMatching {
    /// For each equation index, the matched variable id when one exists.
    pub equation_to_variable: Vec<Option<VariableId>>,
    pub cardinality: usize,
}

impl StructuralMatching {
    pub fn is_square_and_perfect(&self, system: &System) -> bool {
        system.equations.len() == system.variables.len()
            && self.cardinality == system.equations.len()
    }
}

/// Deterministic Hopcroft-Karp maximum bipartite matching over equation-variable incidence.
pub fn maximum_matching(system: &System) -> StructuralMatching {
    let variable_index: BTreeMap<VariableId, usize> = system
        .variables
        .iter()
        .enumerate()
        .map(|(i, v)| (v.id, i))
        .collect();
    let adjacency: Vec<Vec<usize>> = system
        .equations
        .iter()
        .map(|eq| {
            let mut row: Vec<usize> = eq
                .variables
                .iter()
                .filter_map(|id| variable_index.get(id).copied())
                .collect();
            row.sort_unstable();
            row.dedup();
            row
        })
        .collect();

    let left = adjacency.len();
    let right = system.variables.len();
    let mut pair_left = vec![None; left];
    let mut pair_right = vec![None; right];
    let mut distance = vec![usize::MAX; left];
    let mut cardinality = 0;

    loop {
        let mut queue = VecDeque::new();
        for u in 0..left {
            if pair_left[u].is_none() {
                distance[u] = 0;
                queue.push_back(u);
            } else {
                distance[u] = usize::MAX;
            }
        }

        while let Some(u) = queue.pop_front() {
            let next_distance = distance[u].saturating_add(1);
            for &v in &adjacency[u] {
                if let Some(next_u) = pair_right[v] {
                    if distance[next_u] == usize::MAX {
                        distance[next_u] = next_distance;
                        queue.push_back(next_u);
                    }
                }
            }
        }

        let mut augmented = 0;
        for u in 0..left {
            if pair_left[u].is_none()
                && augment(
                    u,
                    &adjacency,
                    &mut pair_left,
                    &mut pair_right,
                    &mut distance,
                )
            {
                augmented += 1;
            }
        }
        if augmented == 0 {
            break;
        }
        cardinality += augmented;
    }

    StructuralMatching {
        equation_to_variable: pair_left
            .into_iter()
            .map(|idx| idx.map(|i| system.variables[i].id))
            .collect(),
        cardinality,
    }
}

fn augment(
    u: usize,
    adjacency: &[Vec<usize>],
    pair_left: &mut [Option<usize>],
    pair_right: &mut [Option<usize>],
    distance: &mut [usize],
) -> bool {
    for &v in &adjacency[u] {
        let can_use = match pair_right[v] {
            None => true,
            Some(next_u) if distance[next_u] == distance[u].saturating_add(1) => {
                augment(next_u, adjacency, pair_left, pair_right, distance)
            }
            Some(_) => false,
        };
        if can_use {
            pair_left[u] = Some(v);
            pair_right[v] = Some(u);
            return true;
        }
    }
    distance[u] = usize::MAX;
    false
}

/// Dependency graph induced by a matching: equation A points to equation B when A uses the
/// variable solved by B. This is the graph on which SCC/BLT decomposition is performed.
pub fn equation_dependency_graph(
    system: &System,
    matching: &StructuralMatching,
) -> Vec<Vec<usize>> {
    let owner: BTreeMap<VariableId, usize> = matching
        .equation_to_variable
        .iter()
        .enumerate()
        .filter_map(|(eq, variable)| variable.map(|v| (v, eq)))
        .collect();
    system
        .equations
        .iter()
        .enumerate()
        .map(|(eq, equation)| {
            let mut deps: Vec<usize> = equation
                .variables
                .iter()
                .filter_map(|v| owner.get(v).copied())
                .filter(|&other| other != eq)
                .collect();
            deps.sort_unstable();
            deps.dedup();
            deps
        })
        .collect()
}

/// Deterministic Tarjan SCC decomposition. Components and members are sorted so persistent
/// compiler artifacts do not depend on hash/random iteration order.
pub fn strongly_connected_components(graph: &[Vec<usize>]) -> Vec<Vec<usize>> {
    struct Tarjan<'a> {
        graph: &'a [Vec<usize>],
        next_index: usize,
        indices: Vec<Option<usize>>,
        lowlink: Vec<usize>,
        stack: Vec<usize>,
        on_stack: Vec<bool>,
        components: Vec<Vec<usize>>,
    }

    impl Tarjan<'_> {
        fn visit(&mut self, v: usize) {
            let index = self.next_index;
            self.next_index += 1;
            self.indices[v] = Some(index);
            self.lowlink[v] = index;
            self.stack.push(v);
            self.on_stack[v] = true;

            let mut neighbors = self.graph[v].clone();
            neighbors.sort_unstable();
            for w in neighbors {
                if self.indices[w].is_none() {
                    self.visit(w);
                    self.lowlink[v] = self.lowlink[v].min(self.lowlink[w]);
                } else if self.on_stack[w] {
                    self.lowlink[v] = self.lowlink[v].min(self.indices[w].expect("visited"));
                }
            }

            if self.lowlink[v] == index {
                let mut component = Vec::new();
                loop {
                    let w = self.stack.pop().expect("SCC root is on stack");
                    self.on_stack[w] = false;
                    component.push(w);
                    if w == v {
                        break;
                    }
                }
                component.sort_unstable();
                self.components.push(component);
            }
        }
    }

    let n = graph.len();
    let mut state = Tarjan {
        graph,
        next_index: 0,
        indices: vec![None; n],
        lowlink: vec![0; n],
        stack: Vec::new(),
        on_stack: vec![false; n],
        components: Vec::new(),
    };
    for v in 0..n {
        if state.indices[v].is_none() {
            state.visit(v);
        }
    }
    state.components.sort_by_key(|c| c[0]);
    state.components
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuralAnalysis {
    pub matching: StructuralMatching,
    pub dependency_graph: Vec<Vec<usize>>,
    pub blocks: Vec<Vec<usize>>,
}

pub fn analyze(system: &System) -> StructuralAnalysis {
    let matching = maximum_matching(system);
    let dependency_graph = equation_dependency_graph(system, &matching);
    let blocks = strongly_connected_components(&dependency_graph);
    StructuralAnalysis {
        matching,
        dependency_graph,
        blocks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EquationId, ExprId};
    use crate::system::{SystemEquation, Variable, VariableKind};

    fn small_system() -> System {
        let x = VariableId(0);
        let y = VariableId(1);
        System {
            name: "two-by-two".into(),
            variables: vec![
                Variable { id: x, name: "x".into(), kind: VariableKind::Algebraic, expression: ExprId(0), derivative_of: None },
                Variable { id: y, name: "y".into(), kind: VariableKind::Algebraic, expression: ExprId(1), derivative_of: None },
            ],
            equations: vec![
                SystemEquation { id: EquationId(0), name: "e0".into(), residual: ExprId(2), variables: vec![x, y] },
                SystemEquation { id: EquationId(1), name: "e1".into(), residual: ExprId(3), variables: vec![y] },
            ],
            ..System::default()
        }
    }

    #[test]
    fn matching_is_perfect_and_deterministic() {
        let system = small_system();
        let a = maximum_matching(&system);
        let b = maximum_matching(&system);
        assert_eq!(a, b);
        assert!(a.is_square_and_perfect(&system));
    }

    #[test]
    fn analysis_builds_dependency_blocks() {
        let analysis = analyze(&small_system());
        assert_eq!(analysis.matching.cardinality, 2);
        assert_eq!(analysis.blocks.iter().map(Vec::len).sum::<usize>(), 2);
    }
}
