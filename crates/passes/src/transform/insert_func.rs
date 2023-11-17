use crate::*;
use tohdl_ir::graph::*;

#[derive(Default)]
pub struct InsertFuncNodes {
    result: TransformResultType,
}



impl InsertFuncNodes {
    /// Get nodes with multiple preds where not all preds are call nodes
    pub(crate) fn get_nodes_muli_preds(&self, graph: &mut CFG) -> Vec<NodeIndex> {
        let candidates = graph
            .nodes()
            .filter(|node| graph.pred(*node).count() > 1)
            .collect::<Vec<_>>();

        // Remove candidates where all preds are call nodes
        candidates
            .into_iter()
            .filter(|node| {
                graph
                    .pred(*node)
                    .map(|pred| match graph.get_node(pred) {
                        Node::Call(_) => true,
                        _ => false,
                    })
                    .any(|x| !x)
            })
            .collect::<Vec<_>>()
    }

    /// Get nodes where pred is a branch node and itself is not a call node
    pub(crate) fn get_nodes_branch_pred(&self, graph: &mut CFG) -> Vec<NodeIndex> {
        let candidates = graph
            .nodes()
            .filter(|node| {
                graph
                    .pred(*node)
                    .map(|pred| match graph.get_node(pred) {
                        Node::Branch(_) => true,
                        _ => false,
                    })
                    .any(|x| x)
            })
            .collect::<Vec<_>>();

        // Remove candidates that are call node
        candidates
            .into_iter()
            .filter(|node| match graph.get_node(*node) {
                Node::Call(_) => false,
                _ => true,
            })
            .collect::<Vec<_>>()
    }
}

impl Transform for InsertFuncNodes {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        // let nodes = self.get_nodes_muli_preds(graph);

        // Get nodes with multiple predicates or a branch as a predicate
        let nodes = self
            .get_nodes_muli_preds(graph)
            .into_iter()
            // .chain(self.get_nodes_branch_pred(graph))
            .collect::<Vec<_>>();

        if nodes.len() > 1 {
            self.result.did_work();
        }

        for node in nodes {
            let preds = graph.pred(node).collect::<Vec<_>>();

            let func_node = graph.add_node(Node::Func(FuncNode { params: vec![] }));
            graph.add_edge(func_node, node, Edge::None);

            for pred in preds {
                let edge_type = graph.rmv_edge(pred, node);
                graph.add_edge(pred, func_node, edge_type);
            }
        }
        &self.result
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::{tests::*, Transform};

    #[test]
    fn main() {
        let mut graph = make_range();

        let mut pass = InsertFuncNodes::default();

        assert_eq!(pass.get_nodes_muli_preds(&mut graph), vec![2.into()]);

        let result = pass.apply(&mut graph);

        println!("result {:?}", result);

        write_graph(&graph, "insert_func_nodes.dot");
    }

    #[test]
    fn branch() {
        let mut graph = make_branch();

        let mut pass = InsertFuncNodes::default();

        pass.apply(&mut graph);

        write_graph(&graph, "insert_func_nodes.dot");
    }
}
