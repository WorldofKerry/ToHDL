use crate::{Transform, TransformResultType};
use tohdl_ir::graph::{AssignNode, BranchNode, Edge, Node, ReturnNode, CFG};

/// Ensures all branch nodes have two branches
/// Adds extra return nodes if this is not the case
#[derive(Debug, Default)]
pub struct ExplicitReturn {
    result: TransformResultType,
}

impl Transform for ExplicitReturn {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        for idx in graph.nodes().collect::<Vec<_>>() {
            // Find leaf nodes that are not return nodes
            if graph.succs(idx).collect::<Vec<_>>().is_empty() {
                if !ReturnNode::downcastable(graph.get_node(idx)) {
                    graph.insert_node_after(ReturnNode { values: vec![] }, idx, Edge::None);
                }
            }
        }
        for idx in graph.nodes().collect::<Vec<_>>() {
            // Find branch nodes with less than two succs
            let succs = graph.succs(idx).collect::<Vec<_>>();
            let len = succs.len();
            if BranchNode::downcastable(graph.get_node(idx)) && len < 2 {
                println!("inside :)");
                match len {
                    0 => {
                        graph.insert_succ(ReturnNode { values: vec![] }, idx, Edge::Branch(true));
                        graph.insert_succ(ReturnNode { values: vec![] }, idx, Edge::Branch(false));
                    }
                    1 => match graph.get_edge(idx, succs[0]).unwrap() {
                        Edge::Branch(true) => {
                            graph.insert_succ(
                                ReturnNode { values: vec![] },
                                idx,
                                Edge::Branch(false),
                            );
                            println!("insert true");
                        }
                        Edge::Branch(false) => {
                            graph.insert_succ(
                                ReturnNode { values: vec![] },
                                idx,
                                Edge::Branch(true),
                            );
                        }
                        _ => unreachable!("Branch has non-branch edge"),
                    },
                    _ => unreachable!("len < 2 is a precondition"),
                }
            }
        }
        &self.result
    }
}
