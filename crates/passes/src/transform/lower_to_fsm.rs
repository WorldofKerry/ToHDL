use std::collections::HashMap;

use crate::*;
use tohdl_ir::expr::VarExpr;
use tohdl_ir::graph::*;

#[derive(Debug, Clone)]
pub struct LowerToFsm {
    // Maps idx (in subgraph) to idx (in original)
    pub subgraph_node_mappings: Vec<Vec<(NodeIndex, NodeIndex)>>,

    // Subgraphs (based on index in Vec)
    pub subgraphs: Vec<CFG>,

    // Maps idx (in original) to subgraph
    pub node_to_subgraph: HashMap<NodeIndex, usize>,

    // Recommended breakpoints (e.g. header of loops)
    pub recommended_breakpoints: Vec<NodeIndex>,

    threshold: usize,
    result: TransformResultType,
}

impl Default for LowerToFsm {
    fn default() -> Self {
        Self {
            threshold: 0, // larger thresholds may invalidate ssa
            result: TransformResultType::default(),
            subgraph_node_mappings: vec![],
            subgraphs: vec![],
            node_to_subgraph: HashMap::new(),
            recommended_breakpoints: vec![],
        }
    }
}

impl LowerToFsm {
    pub fn get_external_funcs(&self, idx: usize) -> HashMap<NodeIndex, usize> {
        let mut external_funcs = HashMap::new();
        for (node_idx, orig_idx) in &self.subgraph_node_mappings[idx] {
            external_funcs.insert(*node_idx, self.node_to_subgraph[orig_idx]);
        }
        external_funcs
    }

    pub fn get_subgraphs(&self) -> &Vec<CFG> {
        &self.subgraphs
    }

    /// After every return or yield node, insert a call node followed by a func node
    /// Ignores return or yield nodes with no successors
    pub(crate) fn split_term_nodes(&self, graph: &mut CFG) {
        for node in graph.nodes() {
            match graph.get_node(node) {
                Node::Return(TermNode { .. }) | Node::Yield(TermNode { .. }) => {
                    let successors: Vec<NodeIndex> = graph.succ(node).collect();

                    if successors.is_empty() {
                        continue;
                    }

                    let call_node = graph.add_node(Node::Call(CallNode { args: vec![] }));
                    let func_node = graph.add_node(Node::Func(FuncNode { params: vec![] }));

                    graph.add_edge(node, call_node, Edge::None);
                    graph.add_edge(call_node, func_node, Edge::None);

                    for successor in &successors {
                        let edge_type = graph.rmv_edge(node, *successor);
                        graph.add_edge(func_node, *successor, edge_type);
                    }
                }
                _ => {}
            }
        }
    }

    /// Make subgraph, where the leaves are either return or yield nodes,
    /// or a call node that has been visited a threshold number of times
    pub(crate) fn recurse(
        &mut self,
        reference_graph: &CFG,
        new_graph: &mut CFG,
        src: NodeIndex,
        visited: HashMap<NodeIndex, usize>,
    ) -> NodeIndex {
        match reference_graph.get_node(src) {
            Node::Return(_) | Node::Yield(_) => {
                let new_node = new_graph.add_node(reference_graph.get_node(src).clone());

                // Recurse on successor, if it exists, and making its visited count infinity
                let successors: Vec<NodeIndex> = reference_graph.succ(src).collect();
                if successors.is_empty() {
                    new_node
                } else {
                    assert_eq!(successors.len(), 1);
                    let successor = successors[0];

                    // Assert is call node
                    match reference_graph.get_node(successor) {
                        Node::Call(_) => {}
                        _ => panic!("successor is not call node"),
                    }

                    let mut new_visited = visited.clone();
                    new_visited.insert(successor, usize::MAX);

                    let new_successor =
                        self.recurse(reference_graph, new_graph, successor, new_visited.clone());
                    new_graph.add_edge(
                        new_node,
                        new_successor,
                        reference_graph.get_edge(src, successor).unwrap().clone(),
                    );

                    match new_graph.get_node(new_successor) {
                        Node::Call(_) => {}
                        _ => panic!("Expected a call node after term node"),
                    }
                    new_node
                }
            }

            Node::Call(_) => {
                let new_node = new_graph.add_node(reference_graph.get_node(src).clone());

                // Check if visited threshold number of times
                let visited_count = visited.get(&src).unwrap_or(&0);
                if visited_count <= &self.threshold {
                    // Recurse
                    let mut new_visited = visited.clone();
                    new_visited.insert(src, visited_count + 1);

                    // Recursively call on successors
                    for successor in reference_graph.succ(src) {
                        let new_succ = self.recurse(
                            reference_graph,
                            new_graph,
                            successor,
                            new_visited.clone(),
                        );
                        new_graph.add_edge(
                            new_node,
                            new_succ,
                            reference_graph.get_edge(src, successor).unwrap().clone(),
                        );
                    }
                } else {
                    let successors = reference_graph.succ(src).collect::<Vec<_>>();
                    assert_eq!(successors.len(), 1);
                    let successor = successors[0];

                    // update global attributes
                    self.subgraph_node_mappings
                        .last_mut()
                        .unwrap()
                        .push((new_node, successor));

                    // Testing with what happens with make_ssa pass applied at successor
                    let mut test_graph = reference_graph.clone();
                    test_graph.set_entry(successor);

                    let test_args =
                        transform::MakeSSA::default().test_rename(&mut test_graph, successor);
                    println!("successor: {}", reference_graph.get_node(successor));
                    println!("test_args: {:#?}", test_args);

                    match new_graph.get_node_mut(new_node) {
                        Node::Call(CallNode { args }) => {
                            for arg in test_args {
                                args.push(arg);
                            }
                        }
                        _ => panic!("Expected call node"),
                    }

                    // Write test graph for debugging
                    test_graph.write_dot("test_graph.dot");
                }
                new_node
            }
            Node::Assign(_) | Node::Branch(_) | Node::Func(_) => {
                let new_node = new_graph.add_node(reference_graph.get_node(src).clone());

                for successor in reference_graph.succ(src) {
                    let new_succ =
                        self.recurse(reference_graph, new_graph, successor, visited.clone());
                    new_graph.add_edge(
                        new_node,
                        new_succ,
                        reference_graph
                            .get_edge(src, successor)
                            .expect(&format!(
                                "{} {} -> {} {}",
                                src,
                                reference_graph.get_node(src),
                                successor.0,
                                reference_graph.get_node(successor)
                            ))
                            .clone(),
                    );
                }

                new_node
            }
        }
    }
}

impl Transform for LowerToFsm {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        self.split_term_nodes(graph);

        // Stores indexes of reference graph that a subgraph needs to be created from
        let mut worklist: Vec<NodeIndex> = vec![];

        worklist.push(graph.get_entry());

        while let Some(node_idx) = worklist.pop() {
            if self.node_to_subgraph.contains_key(&node_idx) {
                continue;
            }

            let mut new_graph = CFG::default();
            self.subgraph_node_mappings.push(vec![]);
            self.recurse(graph, &mut new_graph, node_idx, HashMap::new());

            transform::MakeSSA::transform(&mut new_graph);

            self.node_to_subgraph.insert(node_idx, self.subgraphs.len());
            self.subgraphs.push(new_graph);

            // Update worklist with subgraphs that have not been resolved yet
            if let Some(mappings) = self.subgraph_node_mappings.last() {
                for mapping in mappings {
                    if self.node_to_subgraph.get(&mapping.1).is_some() {
                    } else {
                        worklist.push(mapping.1)
                    }
                }
            }
        }

        &self.result
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::optimize::RemoveRedundantCalls;
    use crate::tests::*;
    use crate::transform::*;

    #[test]
    fn range() {
        let mut graph = make_range();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);
        insert_phi::InsertPhi::default().apply(&mut graph);
        make_ssa::MakeSSA::default().apply(&mut graph);
        // RemoveRedundantCalls::default().apply(&mut graph);

        let mut lower = LowerToFsm::default();
        lower.apply(&mut graph);

        write_graph(&graph, "lower_to_fsm.dot");

        println!("{:#?}", lower);

        // Write all new subgraphs to files
        for (i, subgraph) in lower.subgraphs.iter().enumerate() {
            write_graph(&subgraph, format!("lower_to_fsm_{}.dot", i).as_str());
        }
    }

    #[test]
    fn odd_fib() {
        let mut graph = make_odd_fib();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);
        insert_phi::InsertPhi::default().apply(&mut graph);
        make_ssa::MakeSSA::default().apply(&mut graph);
        // RemoveRedundantCalls::default().apply(&mut graph);

        let mut lower = LowerToFsm::default();
        lower.apply(&mut graph);

        write_graph(&graph, "lower_to_fsm.dot");

        println!("{:#?}", lower);

        // Write all new subgraphs to files
        for (i, subgraph) in lower.subgraphs.iter().enumerate() {
            write_graph(&subgraph, format!("lower_to_fsm_{}.dot", i).as_str());
        }
    }

    #[test]
    fn fib() {
        // let mut graph = make_fib();

        // insert_func::InsertFuncNodes::default().apply(&mut graph);
        // insert_call::InsertCallNodes::default().apply(&mut graph);
        // insert_phi::InsertPhi::default().apply(&mut graph);
        // make_ssa::MakeSSA::default().apply(&mut graph);
        // RemoveRedundantCalls::default().apply(&mut graph);

        // LowerToFsm::default().split_term_nodes(&mut graph);

        // let mut new_graph = DiGraph::default();
        // LowerToFsm::default().recurse(&graph, &mut new_graph, 0.into(), HashMap::new());
        // // graph = new_graph;

        // write_graph(&graph, "lower_to_fsm.dot");
    }

    #[test]
    fn branch() {
        let mut graph = make_branch();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);
        insert_phi::InsertPhi::default().apply(&mut graph);
        make_ssa::MakeSSA::default().apply(&mut graph);
        // RemoveRedundantCalls::default().apply(&mut graph);

        let mut lower = LowerToFsm::default();
        lower.apply(&mut graph);

        write_graph(&graph, "lower_to_fsm.dot");

        println!("{:#?}", lower);

        // Write all new subgraphs to files
        for (i, subgraph) in lower.subgraphs.iter().enumerate() {
            write_graph(&subgraph, format!("lower_to_fsm_{}.dot", i).as_str());
        }
    }
}
