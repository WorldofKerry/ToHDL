use std::cell::RefCell;
use std::collections::HashMap;

use super::*;
use petgraph::stable_graph::IndexType;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

pub struct LowerToFsm {
    lowered_graph: RefCell<DiGraph>,
    threshold: usize,
}

impl LowerToFsm {
    pub fn new() -> Self {
        Self {
            lowered_graph: RefCell::new(DiGraph::new()),
            threshold: 1,
        }
    }

    /// Make subgraph, where the leaves are either return or yield nodes,
    /// or a call node that has been visited a threshold number of times
    pub(crate) fn recurse(
        &self,
        reference_graph: &DiGraph,
        new_graph: &mut DiGraph,
        src: usize,
        visited: HashMap<usize, usize>,
    ) -> usize {
        match reference_graph.get_node(src) {
            Node::Return(_) | Node::Yield(_) => new_graph
                .add_node(reference_graph.get_node(src).clone())
                .index(),

            Node::Call(_) => {
                let new_node = new_graph
                    .add_node(reference_graph.get_node(src).clone())
                    .index();

                // Check if visited threshold number of times
                let visited_count = visited.get(&src).unwrap_or(&0);
                if visited_count <= &self.threshold {
                    // Recurse
                    let mut new_visited = visited.clone();
                    new_visited.insert(src, visited_count + 1);

                    // Recursively call on successors
                    let mut new_successors = vec![];
                    for successor in reference_graph.succ(src) {
                        new_successors.push(self.recurse(
                            reference_graph,
                            new_graph,
                            successor.index(),
                            new_visited.clone(),
                        ));
                    }

                    // Connect new nodes
                    for successor in new_successors {
                        new_graph.add_edge(new_node, successor, Edge::None);
                    }
                }
                new_node.index()
            }
            _ => {
                let new_node = new_graph
                    .add_node(reference_graph.get_node(src).clone())
                    .index();

                let mut new_successors = vec![];
                for successor in reference_graph.succ(src) {
                    new_successors.push(self.recurse(
                        reference_graph,
                        new_graph,
                        successor.index(),
                        visited.clone(),
                    ));
                }

                for successor in new_successors {
                    new_graph.add_edge(new_node, successor, Edge::None);
                }

                new_node
            }
        }
    }
}

impl Transform for LowerToFsm {
    fn transform(&self, graph: &mut DiGraph) {}
}

#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use super::*;

    #[test]
    fn range() {
        let mut graph = make_range();

        insert_func::InsertFuncNodes {}.transform(&mut graph);
        insert_call::InsertCallNodes {}.transform(&mut graph);
        insert_phi::InsertPhi {}.transform(&mut graph);
        make_ssa::MakeSSA::new().transform(&mut graph);

        write_graph(&graph, "lower_to_fsm.dot");
    }

    #[test]
    fn fib() {
        let mut graph = make_fib();

        insert_func::InsertFuncNodes {}.transform(&mut graph);
        insert_call::InsertCallNodes {}.transform(&mut graph);
        insert_phi::InsertPhi {}.transform(&mut graph);
        make_ssa::MakeSSA::new().transform(&mut graph);

        let mut new_graph = DiGraph::new();
        LowerToFsm::new().recurse(&graph, &mut new_graph, 0, HashMap::new());
        graph = new_graph;

        write_graph(&graph, "lower_to_fsm.dot");
    }
}
