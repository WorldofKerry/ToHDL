use tohdl_ir::graph::{CallNode, FuncNode, Node, NodeIndex};

use crate::*;

pub struct RemoveRedundantCalls {
    result: TransformResultType,
}

impl Default for RemoveRedundantCalls {
    fn default() -> Self {
        Self {
            result: TransformResultType::default(),
        }
    }
}

impl Transform for RemoveRedundantCalls {
    fn apply(&mut self, graph: &mut DiGraph) -> &TransformResultType {
        &self.result
    }
}

impl RemoveRedundantCalls {
    /// Find call nodes with no arguments
    pub(crate) fn get_argless_funcs(&self, graph: &mut DiGraph) -> Vec<NodeIndex> {
        graph
            .nodes()
            .filter(|node| match graph.get_node(*node) {
                Node::Call(CallNode { args }) => args.len() == 0,
                _ => false,
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::{manager::PassManager, optimize::*, tests::*, transform::*, Transform};

    #[test]
    fn main() {
        let mut graph = make_fib();
        let mut manager = PassManager::default();

        manager.add_pass(InsertFuncNodes::transform);
        manager.add_pass(InsertCallNodes::transform);
        manager.add_pass(InsertPhi::transform);
        manager.add_pass(MakeSSA::transform);

        manager.apply(&mut graph);

        let mut pass = RemoveRedundantCalls::default();

        assert_eq!(
            pass.get_argless_funcs(&mut graph),
            vec![16.into(), 17.into()]
        );

        graph.write_dot("remove_redundant_calls.dot");
    }
}
