use crate::{BasicTransform, TransformResultType};
use tohdl_ir::graph::{AssignNode, BranchEdge, BranchNode, Node, NoneEdge, ReturnNode, CFG};

/// Ensures all leaf nodes are return nodes
#[derive(Debug, Default)]
pub struct ExplicitReturn {
    result: TransformResultType,
}

impl BasicTransform for ExplicitReturn {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        for idx in graph.nodes().collect::<Vec<_>>() {
            // Find leaf nodes that are not return nodes
            if graph.succs(idx).collect::<Vec<_>>().is_empty() {
                if !ReturnNode::downcastable(graph.get_node(idx)) {
                    graph.insert_node_after(ReturnNode { values: vec![] }, idx, NoneEdge.into());
                }
            }
        }
        &self.result
    }
}
