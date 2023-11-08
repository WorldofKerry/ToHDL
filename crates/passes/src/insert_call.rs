use super::*;
use tohdl_ir::graph::*;

pub struct InsertCallNodes {}

impl InsertCallNodes {}

impl Transform for InsertCallNodes {
    fn transform(&self, graph: &mut DiGraph) {
        for node in graph.nodes() {
            let node_data = graph.get_node(node);
            match node_data {
                Node::Func(_) => {
                    let preds = graph.pred(node).collect::<Vec<_>>();
                    for pred in preds {
                        let pred_data = graph.get_node(pred);
                        match pred_data {
                            Node::Call(_) => {
                                // panic!("Call node cannot be a predecessor of a Func node")
                            }
                            _ => {
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
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use super::*;

    #[test]
    fn main() {
        let mut graph = make_range();
        let mut graph_copy = make_range();
        insert_func::InsertFuncNodes {}.transform(&mut graph_copy);
        InsertCallNodes {}.transform(&mut graph_copy);

        // Graphs should be equal even with infinite number of these transforms in any order
        insert_func::InsertFuncNodes {}.transform(&mut graph);
        InsertCallNodes {}.transform(&mut graph);
        insert_func::InsertFuncNodes {}.transform(&mut graph);
        insert_func::InsertFuncNodes {}.transform(&mut graph);
        insert_func::InsertFuncNodes {}.transform(&mut graph);
        InsertCallNodes {}.transform(&mut graph);
        InsertCallNodes {}.transform(&mut graph);
        InsertCallNodes {}.transform(&mut graph);
        insert_func::InsertFuncNodes {}.transform(&mut graph);
        InsertCallNodes {}.transform(&mut graph);
        InsertCallNodes {}.transform(&mut graph);
        insert_func::InsertFuncNodes {}.transform(&mut graph);
        insert_func::InsertFuncNodes {}.transform(&mut graph);
        InsertCallNodes {}.transform(&mut graph);
        InsertCallNodes {}.transform(&mut graph);

        assert_eq!(graph, graph_copy);

        write_graph(&graph, "insert_call.dot");
    }
}
