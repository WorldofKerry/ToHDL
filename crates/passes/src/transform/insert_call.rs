use crate::*;
use tohdl_ir::graph::*;

#[derive(Default)]
pub struct InsertCallNodes {
    result: TransformResultType,
}



impl Transform for InsertCallNodes {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        let nodes = graph.nodes().collect::<Vec<_>>();
        for node in nodes {
            let node_data = graph.get_node(node);
            match node_data {
                Node::Func(_) => {
                    let preds = graph.pred(node).collect::<Vec<_>>();
                    for pred in preds {
                        let pred_data = graph.get_node(pred);
                        match pred_data {
                            Node::Call(_) => {}
                            _ => {
                                self.result.did_work();
                                let call_node =
                                    graph.add_node(Node::Call(CallNode { args: vec![] }));

                                let edge_type = graph.rmv_edge(pred, node);
                                graph.add_edge(pred, call_node, edge_type);
                                graph.add_edge(call_node, node, Edge::None);
                            }
                        }
                    }
                }
                _ => {}
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
        let mut graph_copy = make_range();
        InsertFuncNodes::default().apply(&mut graph_copy);
        InsertCallNodes::default().apply(&mut graph_copy);

        // Graphs should be equal even with infinite number of these transforms in any order
        insert_func::InsertFuncNodes::default().apply(&mut graph);
        InsertCallNodes::default().apply(&mut graph);
        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_func::InsertFuncNodes::default().apply(&mut graph);
        InsertCallNodes::default().apply(&mut graph);
        InsertCallNodes::default().apply(&mut graph);
        InsertCallNodes::default().apply(&mut graph);
        insert_func::InsertFuncNodes::default().apply(&mut graph);
        InsertCallNodes::default().apply(&mut graph);
        InsertCallNodes::default().apply(&mut graph);
        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_func::InsertFuncNodes::default().apply(&mut graph);
        InsertCallNodes::default().apply(&mut graph);
        InsertCallNodes::default().apply(&mut graph);

        assert!(tohdl_ir::graph::CFG::graph_eq(&graph, &graph_copy));

        write_graph(&graph, "insert_call.dot");
    }
}
