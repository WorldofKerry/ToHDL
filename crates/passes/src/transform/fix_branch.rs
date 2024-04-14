use crate::{BasicTransform, TransformResultType};
use tohdl_ir::graph::{AssignNode, BranchEdge, BranchNode, Node, ReturnNode, CFG};

/// Ensures all branch nodes have two branches
/// Adds extra return nodes if this is not the case
#[derive(Debug, Default)]
pub struct FixBranch {
    result: TransformResultType,
}

impl BasicTransform for FixBranch {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        for idx in graph.nodes().collect::<Vec<_>>() {
            // Find branch nodes with less than two succs
            let succs = graph.succs(idx).collect::<Vec<_>>();
            if BranchNode::downcastable(graph.get_node(idx)) {
                match succs.len() {
                    0 => {
                        // No child edges: insert both a true edge and a false edge
                        graph.insert_succ(
                            ReturnNode { values: vec![] },
                            idx,
                            BranchEdge::new(true).into(),
                        );
                        graph.insert_succ(
                            ReturnNode { values: vec![] },
                            idx,
                            BranchEdge::new(false).into(),
                        );
                    }
                    1 => {
                        // One child edge: insert the missing one
                        if let Some(BranchEdge { condition }) = graph
                            .get_edge(idx, succs[0])
                            .unwrap()
                            .downcast_ref::<BranchEdge>()
                        {
                            graph.insert_succ(
                                ReturnNode { values: vec![] },
                                idx,
                                BranchEdge::new(!condition).into(),
                            );
                        }
                    }
                    2 => {}
                    _ => {
                        panic!("More than 2 children for branch node")
                    }
                }
            }
        }
        &self.result
    }
}
