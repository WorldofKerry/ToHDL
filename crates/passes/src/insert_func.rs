use super::*;
use tohdl_ir::graph::*;

pub struct InsertFuncNodes {}

impl InsertFuncNodes {
    pub(crate) fn get_nodes_muli_succs(&self, graph: &mut DiGraph) -> Vec<usize> {
        graph
            .nodes()
            .filter(|node| graph.succ(*node).count() > 1)
            .collect::<Vec<_>>()
    }
}

impl Transform for InsertFuncNodes {
    fn transform(&self, graph: &mut DiGraph) {
        let nodes = self.get_nodes_muli_succs(graph);

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

        let pass = InsertFuncNodes {};

        assert_eq!(pass.get_nodes_muli_succs(&mut graph), vec![1]);

        let result = pass.transform(&mut graph);

        println!("result {:?}", result);

        write_graph(&graph, "insert_func_nodes.dot");
    }
}
