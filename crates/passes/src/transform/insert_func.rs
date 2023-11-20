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
        
        candidates
            .into_iter()
            .filter(|idx| 
                // Remove candidates where all preds are call nodes
                !graph
                    .pred(*idx)
                    .all(|idx| CallNode::filter(graph.get_node(idx)))
            )
            .collect::<Vec<_>>()
    }
}

impl Transform for InsertFuncNodes {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        // Get nodes with multiple predicates or a branch as a predicate
        let nodes = self
            .get_nodes_muli_preds(graph);

        if nodes.len() > 1 {
            self.result.did_work();
        }

        for node in nodes {
            let preds = graph.pred(node).collect::<Vec<_>>();

            let func_node = graph.add_node(FuncNode { params: vec![] });
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
    fn range() {
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
        
        assert_eq!(pass.get_nodes_muli_preds(&mut graph), vec![4.into()]);

        pass.apply(&mut graph);

        write_graph(&graph, "insert_func_nodes.dot");
    }
}
