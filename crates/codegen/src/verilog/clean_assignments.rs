use tohdl_ir::graph::{AssignNode, Node, CFG};
use tohdl_passes::{Transform, TransformResultType};

use super::memory::LoadNode;

#[derive(Default)]
pub struct RemoveAssignNodes {
    result: TransformResultType,
}

impl Transform for RemoveAssignNodes {
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
