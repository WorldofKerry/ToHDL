use tohdl_ir::graph::{AssignNode, NodeLike, CFG};
use tohdl_passes::{Transform, TransformResultType};

use super::memory::MemoryNode;

#[derive(Default)]
pub struct RemoveAssignNodes {
    result: TransformResultType,
}

impl Transform for RemoveAssignNodes {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        for idx in graph.nodes().collect::<Vec<_>>() {
            if let Some(MemoryNode { lvalue, rvalue }) = MemoryNode::concrete(graph.get_node(idx)) {
                graph.replace_node(
                    idx,
                    AssignNode {
                        lvalue: lvalue.clone(),
                        rvalue: rvalue.clone(),
                    },
                );
            }
        }
        &self.result
    }
}
