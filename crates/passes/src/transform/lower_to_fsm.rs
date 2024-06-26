use std::collections::BTreeMap;

use crate::transform::BraunEtAl;
use crate::*;
use tohdl_ir::expr::VarExpr;
use tohdl_ir::graph::*;

#[derive(Clone)]
pub struct LowerToFsm {
    // Maps idx (in subgraph) to idx (in original)
    pub subgraph_node_mappings: Vec<Vec<(NodeIndex, NodeIndex)>>,

    // Subgraphs (based on index in Vec)
    pub subgraphs: Vec<CFG>,

    // Maps idx (in original) to subgraph
    pub node_to_subgraph: BTreeMap<NodeIndex, usize>,

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
            threshold: 0, // larger thresholds may invalidate ssa
            result: TransformResultType::default(),
            subgraph_node_mappings: vec![],
            subgraphs: vec![],
            node_to_subgraph: BTreeMap::new(),
            recommended_breakpoints: vec![],
            call_node_before_yield: vec![],
        }
    }
}

impl LowerToFsm {
    /// Get a mapping of node index to subgraph
    pub fn get_external_funcs(&self, idx: usize) -> BTreeMap<NodeIndex, usize> {
        let mut external_funcs = BTreeMap::new();
        for (node_idx, orig_idx) in &self.subgraph_node_mappings[idx] {
            external_funcs.insert(*node_idx, self.node_to_subgraph[orig_idx]);
        }
        external_funcs
    }

    pub fn get_subgraphs(&self) -> &Vec<CFG> {
        &self.subgraphs
    }

    /// Before every return or yield node, insert a call node followed by a func node
    /// Returns vec of node indexes of inserted call nodes
    pub(crate) fn before_yield_nodes(&self, graph: &mut CFG) -> Vec<NodeIndex> {
        let mut added_call_nodes = vec![];
        for node in graph.nodes() {
            if ReturnNode::downcastable(graph.get_node(node))
                || YieldNode::downcastable(graph.get_node(node))
            {
                let preds = graph.preds(node).collect::<Vec<NodeIndex>>();

                let call_node = graph.add_node(CallNode { args: vec![] });
                let func_node = graph.add_node(FuncNode { params: vec![] });

                added_call_nodes.push(call_node);

                graph.add_edge(call_node, func_node, NoneEdge.into());
                graph.add_edge(func_node, node, NoneEdge.into());

                for pred in &preds {
                    let edge_type = graph.rmv_edge(*pred, node);
                    graph.add_edge(*pred, call_node, edge_type);
                }
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
        visited: BTreeMap<NodeIndex, usize>,
    ) -> NodeIndex {
        let node_data = reference_graph.get_node(src);
        // reference_graph.write_dot("output.dot");
        if let Some(term) = ReturnNode::concrete(node_data) {
            let new_node = new_graph.add_node(term.clone());

            let mut new_visited = visited.clone();

            self.mark_call_before_term(&mut new_visited);

            for successor in reference_graph.succs(src) {
                let new_succ =
                    self.recurse(reference_graph, new_graph, successor, new_visited.clone());
                new_graph.add_edge(
                    new_node,
                    new_succ,
                    reference_graph
                        .get_edge(src, successor)
                        .unwrap_or_else(|| {
                            panic!(
                                "{} {} -> {} {}",
                                src,
                                reference_graph.get_node(src),
                                successor.0,
                                reference_graph.get_node(successor)
                            )
                        })
                        .clone(),
                );
            }

            new_node
        } else if let Some(term) = YieldNode::concrete(node_data) {
            let new_node = new_graph.add_node(term.clone());

            let mut new_visited = visited.clone();

            self.mark_call_before_term(&mut new_visited);

            for successor in reference_graph.succs(src) {
                let new_succ =
                    self.recurse(reference_graph, new_graph, successor, new_visited.clone());
                new_graph.add_edge(
                    new_node,
                    new_succ,
                    reference_graph
                        .get_edge(src, successor)
                        .unwrap_or_else(|| {
                            panic!(
                                "{} {} -> {} {}",
                                src,
                                reference_graph.get_node(src),
                                successor.0,
                                reference_graph.get_node(successor)
                            )
                        })
                        .clone(),
                );
            }

            new_node
        } else if let Some(call) = CallNode::concrete(node_data) {
            let new_node = new_graph.add_node(call.clone());

            // Check if visited threshold number of times
            let visited_count = visited.get(&src).unwrap_or(&0);
            if visited_count <= &self.threshold {
                // Recurse
                let mut new_visited = visited.clone();
                new_visited.insert(src, visited_count + 1);

                // Recursively call on successors
                for successor in reference_graph.succs(src) {
                    let new_succ =
                        self.recurse(reference_graph, new_graph, successor, new_visited.clone());
                    new_graph.add_edge(
                        new_node,
                        new_succ,
                        reference_graph.get_edge(src, successor).unwrap().clone(),
                    );
                }
            } else {
                // println!("broke here {} {:?}", src, visited);
                let successors = reference_graph.succs(src).collect::<Vec<_>>();
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

                let result = BraunEtAl::find_external_vars(&mut test_graph.clone(), successor);
                // println!("extern vars result {:?}", result);
                // println!("successor: {}", reference_graph.get_node(successor));

                match CallNode::concrete_mut(new_graph.get_node_mut(new_node)) {
                    Some(CallNode { args }) => {
                        for arg in result {
                            args.push(arg.clone());
                        }
                        // println!("da_args {args:?}")
                    }
                    _ => panic!("Expected call node"),
                }


                // Write test graph for debugging
                // test_graph.write_dot("test_graph.dot");
            }
            new_node
        } else {
            let new_node = new_graph.add_node_boxed(dyn_clone::clone_box(&**node_data));
            for successor in reference_graph.succs(src) {
                let new_succ = self.recurse(reference_graph, new_graph, successor, visited.clone());
                new_graph.add_edge(
                    new_node,
                    new_succ,
                    reference_graph
                        .get_edge(src, successor)
                        .unwrap_or_else(|| {
                            panic!(
                                "{} {} -> {} {}",
                                src,
                                reference_graph.get_node(src),
                                successor.0,
                                reference_graph.get_node(successor)
                            )
                        })
                        .clone(),
                );
            }

            new_node
        }
    }

    /// Create a default visited
    fn create_default_visited(&self) -> BTreeMap<NodeIndex, usize> {
        // make all recommended breakpoints infinite
        let mut map = BTreeMap::new();
        for recommended_breakpoint in &self.recommended_breakpoints {
            map.insert(*recommended_breakpoint, usize::MAX);
        }
        map
    }

    /// Mark preds of yield nodes
    fn mark_call_before_term(&self, visited: &mut BTreeMap<NodeIndex, usize>) {
        for node in &self.call_node_before_yield {
            visited.insert(*node, usize::MAX);
        }
    }
}

impl BasicTransform for LowerToFsm {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        let loops = algorithms::loop_detector::detect_loops(graph);

        // Get all atches as recommended breakpoints
        let recommended_breakpoints = loops
            .iter()
            .flat_map(|loop_| loop_.latches.clone())
            .collect::<Vec<_>>();

        // println!("recommended_breakpoints: {:#?}", recommended_breakpoints);

        self.recommended_breakpoints = recommended_breakpoints;

        self.call_node_before_yield = self.before_yield_nodes(graph);

        // println!("call before yield {:?}", self.call_node_before_yield);

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

            transform::BraunEtAl::transform(&mut new_graph);

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
    use crate::tests::*;
    use crate::transform::*;

    #[test]
    fn range() {
        let mut graph = make_range();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);
        // insert_phi::InsertPhi::default().apply(&mut graph);
        // make_ssa::MakeSSA::default().apply(&mut graph);
        // RemoveRedundantCalls::default().apply(&mut graph);
        transform::BraunEtAl::transform(&mut graph);

        let mut lower = LowerToFsm::default();
        lower.apply(&mut graph);

        // graph.write_dot("lower_to_fsm.dot");

        // Write all new subgraphs to files
        for (i, subgraph) in lower.subgraphs.iter().enumerate() {
            // subgraph.write_dot(format!("lower_to_fsm_{}.dot", i).as_str());
        }
    }

    #[test]
    fn odd_fib() {
        let mut graph = make_even_fib();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);
        transform::BraunEtAl::transform(&mut graph);

        let mut lower = LowerToFsm::default();
        lower.apply(&mut graph);

        // graph.write_dot("lower_to_fsm.dot");

        // Write all new subgraphs to files
        for (i, subgraph) in lower.subgraphs.iter().enumerate() {
            // subgraph.write_dot(format!("lower_to_fsm_{}.dot", i).as_str());
        }
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

        // graph.write_dot("lower_to_fsm.dot");

        // Write all new subgraphs to files
        for (i, subgraph) in lower.subgraphs.iter().enumerate() {
            // subgraph.write_dot(format!("lower_to_fsm_{}.dot", i).as_str());
        }
    }
}
