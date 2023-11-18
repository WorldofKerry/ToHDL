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

    // Call nodes immediately before yield nodes
    pub call_node_before_yield: Vec<NodeIndex>,

    threshold: usize,
    result: TransformResultType,
}

impl Default for LowerToFsm {
    fn default() -> Self {
        Self {
            threshold: 1, // larger thresholds may invalidate ssa
            result: TransformResultType::default(),
            subgraph_node_mappings: vec![],
            subgraphs: vec![],
            node_to_subgraph: HashMap::new(),
            recommended_breakpoints: vec![],
            call_node_before_yield: vec![],
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

    /// Before every return or yield node, insert a call node followed by a func node
    /// Returns vec of node indexes of inserted call nodes
    pub(crate) fn before_yield_nodes(&self, graph: &mut CFG) -> Vec<NodeIndex> {
        let mut added_call_nodes = vec![];
        for node in graph.nodes() {
            match graph.get_node(node) {
                Node::Yield(TermNode { .. }) => {
                    let preds = graph.pred(node).collect::<Vec<NodeIndex>>();

                    let call_node = graph.add_node(Node::Call(CallNode { args: vec![] }));
                    let func_node = graph.add_node(Node::Func(FuncNode { params: vec![] }));

                    added_call_nodes.push(call_node);

                    graph.add_edge(call_node, func_node, Edge::None);
                    graph.add_edge(func_node, node, Edge::None);

                    for pred in &preds {
                        let edge_type = graph.rmv_edge(*pred, node);
                        graph.add_edge(*pred, call_node, edge_type);
                    }
                }
                _ => {}
            }
        }
        added_call_nodes
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

                let mut new_visited = visited.clone();

                self.mark_call_before_term(&mut new_visited);

                for successor in reference_graph.succ(src) {
                    let new_succ =
                        self.recurse(reference_graph, new_graph, successor, new_visited.clone());
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
                    println!("broke here {} {:?}", src, visited);
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
                            // let len_diff = test_args.len() as isize - args.len() as isize;
                            // if len_diff > 0 {
                            //     let len_diff = len_diff as usize;
                            //     for arg in &test_args[test_args.len() - len_diff..] {
                            //         args.push(arg.clone());
                            //     }
                            // }
                            for arg in test_args {
                                args.push(arg.clone());
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

    /// Create a default visited
    fn create_default_visited(&self) -> HashMap<NodeIndex, usize> {
        // make all recommended breakpoints infinite
        let mut hashmap = HashMap::new();
        for recommended_breakpoint in &self.recommended_breakpoints {
            hashmap.insert(*recommended_breakpoint, usize::MAX);
        }
        hashmap
    }

    /// Mark preds of yield nodes
    fn mark_call_before_term(&self, visited: &mut HashMap<NodeIndex, usize>) {
        for node in &self.call_node_before_yield {
            visited.insert(*node, usize::MAX);
        }
    }
}

impl Transform for LowerToFsm {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        let loops = algorithms::loop_detector::detect_loops(&graph);

        // Get all atches as recommended breakpoints
        let recommended_breakpoints = loops
            .iter()
            .flat_map(|loop_| loop_.latches.clone())
            .collect::<Vec<_>>();

        println!("recommended_breakpoints: {:#?}", recommended_breakpoints);

        self.recommended_breakpoints = recommended_breakpoints;

        self.call_node_before_yield = self.before_yield_nodes(graph);

        println!("call before yield {:?}", self.call_node_before_yield);

        // Stores indexes of reference graph that a subgraph needs to be created from
        let mut worklist: Vec<NodeIndex> = vec![];

        worklist.push(graph.get_entry());

        while let Some(node_idx) = worklist.pop() {
            if self.node_to_subgraph.contains_key(&node_idx) {
                continue;
            }

            let mut new_graph = CFG::default();
            self.subgraph_node_mappings.push(vec![]);
            self.recurse(
                graph,
                &mut new_graph,
                node_idx,
                self.create_default_visited(),
            );

            // new_graph.write_dot("test_graph.dot");
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

        // Write all new subgraphs to files
        for (i, subgraph) in lower.subgraphs.iter().enumerate() {
            write_graph(&subgraph, format!("lower_to_fsm_{}.dot", i).as_str());
        }
    }

    #[test]
    fn odd_fib() {
        let mut graph = make_even_fib();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);
        insert_phi::InsertPhi::default().apply(&mut graph);
        make_ssa::MakeSSA::default().apply(&mut graph);
        // RemoveRedundantCalls::default().apply(&mut graph);

        let mut lower = LowerToFsm::default();
        lower.apply(&mut graph);

        write_graph(&graph, "lower_to_fsm.dot");

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
