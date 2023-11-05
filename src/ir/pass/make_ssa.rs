use super::*;
use crate::ir::graph::*;

pub struct MakeSSA {}

impl MakeSSA {
    /// Gets block of statements
    pub(crate) fn block(&self, graph: &DiGraph, node: usize) -> Vec<usize> {
        return graph.dfs(node, &|n| match n {
            Node::Call(_) => false,
            _ => true,
        });
    }
}

impl Transform for MakeSSA {
    fn transform(&self, graph: &mut DiGraph) {
        for node in graph.nodes() {
            let node_data = graph.get_node(node);
            match node_data {
                Node::Func(_) => {
                    let preds = graph.preds(node).collect::<Vec<_>>();
                    for pred in preds {
                        let pred_data = graph.get_node(pred);
                        match pred_data {
                            Node::Call(_) => {
                                panic!("Call node cannot be a predecessor of a Func node")
                            }
                            _ => {
                                let call_node =
                                    graph.add_node(Node::Call(CallNode { params: vec![] }));

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
    use super::*;
    use crate::ir::tests::*;

    #[test]
    fn main() {
        let mut graph = make_range();

        insert_func::InsertFuncNodes {}.transform(&mut graph);
        MakeSSA {}.transform(&mut graph);

        let result = MakeSSA {}.block(&graph, 5);

        println!("result {:?}", result);

        write_graph(&graph, "make_ssa.dot");
    }
}
