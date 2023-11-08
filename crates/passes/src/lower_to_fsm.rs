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

    /// After every return or yield node, insert a call node followed by a func node
    /// Ignores return or yield nodes with no successors
    pub(crate) fn split_term_nodes(&self, graph: &mut DiGraph) {
        for node in graph.nodes() {
            match graph.get_node(node) {
                Node::Return(TermNode { .. }) | Node::Yield(TermNode { .. }) => {
                    let successors: Vec<usize> = graph.succ(node).map(|x| x.index()).collect();

                    if successors.len() == 0 {
                        continue;
                    }

                    let call_node = graph.add_node(Node::Call(CallNode { args: vec![] }));
                    let func_node = graph.add_node(Node::Func(FuncNode { params: vec![] }));

                    graph.add_edge(node, call_node, Edge::None);
                    graph.add_edge(call_node, func_node, Edge::None);

                    for successor in &successors {
                        graph.rmv_edge(node, *successor);
                    }
                    for successor in &successors {
                        graph.add_edge(func_node, *successor, Edge::None);
                    }
                }
                _ => {}
            }
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
            Node::Return(_) | Node::Yield(_) => {
                let new_node = new_graph
                    .add_node(reference_graph.get_node(src).clone())
                    .index();

                // Recurse on successor, if it exists, and making its visited count infinity
                let successors: Vec<usize> = reference_graph.succ(src).collect();
                if successors.len() == 0 {
                    new_node
                } else {
                    let successor = successors[0];

                    // Assert is call node
                    match reference_graph.get_node(successor) {
                        Node::Call(_) => {}
                        _ => panic!("successor is not call node"),
                    }

                    let mut new_visited = visited.clone();
                    new_visited.insert(successor, usize::MAX);

                    let new_successor = self.recurse(
                        reference_graph,
                        new_graph,
                        successor.index(),
                        new_visited.clone(),
                    );
                    new_graph.add_edge(new_node, new_successor, Edge::None);
                    new_node
                }
            }

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

        LowerToFsm::new().split_term_nodes(&mut graph);

        let mut new_graph = DiGraph::new();
        LowerToFsm::new().recurse(&graph, &mut new_graph, 0, HashMap::new());
        graph = new_graph;

        write_graph(&graph, "lower_to_fsm.dot");
    }
}
