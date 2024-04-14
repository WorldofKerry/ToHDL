use tohdl_ir::graph::{AssignNode, Node, CFG};
use tohdl_passes::{BasicTransform, TransformResultType};

use super::memory::LoadNode;

#[derive(Default)]
pub struct RemoveAssignNodes {
    result: TransformResultType,
}

impl BasicTransform for RemoveAssignNodes {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        for idx in graph.nodes().collect::<Vec<_>>() {
            if let Some(LoadNode { lvalue, rvalue }) = LoadNode::concrete(graph.get_node(idx)) {
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
