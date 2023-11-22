use tohdl_ir::graph::{FuncNode, Node, NodeIndex};

use crate::*;

#[derive(Default)]
pub struct RemoveRedundantCalls {
    result: TransformResultType,
}

impl Transform for RemoveRedundantCalls {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        self.remove_call_node(graph);
        &self.result
    }
}

impl RemoveRedundantCalls {
    /// Finds all func nodes with no args and at least one pred
    pub(crate) fn get_paramless_funcs_with_succs(&self, graph: &mut CFG) -> Vec<NodeIndex> {
        // graph
        //     .nodes()
        //     .filter(|node| match graph.get_node(*node) {
        //         Node::Func(FuncNode { params }) => {
        //             params.is_empty() && graph.pred(*node).count() > 0
        //         }
        //         _ => false,
        //     })
        //     .collect::<Vec<_>>()
        todo!()
    }

    /// Remove call node and func node associated with it and its predecessors
    pub(crate) fn remove_call_node(&mut self, graph: &mut CFG) {
        let mut call_nodes = self.get_paramless_funcs_with_succs(graph);
        while let Some(node) = call_nodes.pop() {
            self.result.did_work();
            let preds = graph.pred(node).collect::<Vec<_>>();
            for pred in preds {
                graph.rmv_node_and_reattach(pred);
            }
            graph.rmv_node_and_reattach(node)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::{manager::PassManager, tests::*, transform::*, Transform};

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

        // assert_eq!(
        //     pass.get_paramless_funcs_with_succs(&mut graph),
        //     vec![12.into(), 13.into()]
        // );

        pass.remove_call_node(&mut graph);

        graph.write_dot("remove_redundant_calls.dot");
    }
}
