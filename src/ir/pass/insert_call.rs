use super::*;
use crate::ir::graph::*;

pub struct InsertCallNodes {}

impl InsertCallNodes {
    // pub(crate) fn get_nodes_muli_succs(&self, graph: &mut DiGraph) -> Vec<usize> {
    //     // graph
    //     //     .nodes()
    //     //     .filter(|node| graph.succ(*node).count() > 1)
    //     //     .collect::<Vec<_>>()
    // }
}

impl Transform for InsertCallNodes {
    fn transform(&self, graph: &mut DiGraph) {
        for node in graph.nodes() {
            let node_data = graph.get_node(node);
            match node_data {
                Node::Func(_) => {
                    let preds = graph.preds(node).collect::<Vec<_>>();
                    for pred in preds {
                        let call_node = graph.add_node(Node::Call(CallNode { params: vec![] }));

                        let edge_type = graph.rmv_edge(pred, node);
                        graph.add_edge(pred, call_node, edge_type);
                        graph.add_edge(call_node, node, Edge::None);
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::tests::*;

    #[test]
    fn main() {
        let mut graph = make_range();

        insert_func::InsertFuncNodes {}.transform(&mut graph);
        let result = InsertCallNodes {}.transform(&mut graph);

        println!("result {:?}", result);

        write_graph(&graph, "insert_call.dot");
    }
}
