use crate::{Transform, TransformResultType};
use tohdl_ir::graph::{AssignNode, BranchNode, Edge, Node, ReturnNode, CFG};

/// Ensures all leaf nodes are return nodes
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
        &self.result
    }
}
