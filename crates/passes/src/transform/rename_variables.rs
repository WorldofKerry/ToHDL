use crate::*;
use tohdl_ir::graph::*;

#[derive(Default)]
pub struct RenameVariables {
    result: TransformResultType,
}

impl ContextfulTransfrom<String> for RenameVariables {
    fn apply_contextful(&mut self, graph: &mut CFG, context: &mut String) -> &TransformResultType {
        let idxes = graph.nodes().collect::<Vec<_>>();
        for idx in idxes {
            let node = graph.get_node_mut(idx);
            for var in node.declared_vars_mut() {
                var.name = format!("{}_{}", context, var.name);
            }
            for var in node.referenced_vars_mut() {
                var.name = format!("{}_{}", context, var.name);
            }
        }
        &self.result
    }
}
