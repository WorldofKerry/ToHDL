use super::*;
use tohdl_ir::graph::*;

pub struct InsertFuncNodes {}

impl InsertFuncNodes {
    pub fn new() -> Self {
        Self {}
    }

    pub fn boxed() -> Box<dyn Transform> {
        Box::new(Self::new())
    }

    /// Get nodes with multiple preds where not all preds are call nodes
    pub(crate) fn get_nodes_muli_preds(&self, graph: &mut DiGraph) -> Vec<usize> {
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
    pub(crate) fn get_nodes_branch_pred(&self, graph: &mut DiGraph) -> Vec<usize> {
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
    fn transform(&mut self, graph: &mut DiGraph) {
        // let nodes = self.get_nodes_muli_preds(graph);

        // Combine two vecs
        let nodes = self
            .get_nodes_muli_preds(graph)
            .into_iter()
            .chain(self.get_nodes_branch_pred(graph).into_iter())
            .collect::<Vec<_>>();

        println!("Inserting Func node at {:?}", nodes);
        for node in nodes {
            let preds = graph.pred(node).collect::<Vec<_>>();

            let func_node = graph.add_node(Node::Func(FuncNode { params: vec![] }));
            graph.add_edge(func_node, node, Edge::None);

            for pred in preds {
                let edge_type = graph.rmv_edge(pred, node);
                graph.add_edge(pred, func_node, edge_type);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use super::*;

    #[test]
    fn main() {
        let mut graph = make_range();

        let mut pass = InsertFuncNodes {};

        assert_eq!(pass.get_nodes_muli_preds(&mut graph), vec![2]);

        let result = pass.transform(&mut graph);

        println!("result {:?}", result);

        write_graph(&graph, "insert_func_nodes.dot");
    }

    #[test]
    fn branch() {
        let mut graph = make_branch();

        let mut pass = InsertFuncNodes {};

        pass.transform(&mut graph);

        write_graph(&graph, "insert_func_nodes.dot");
    }
}
