use crate::*;
use tohdl_ir::graph::*;

#[derive(Default)]
pub struct InsertCallNodes {
    result: TransformResultType,
}

impl BasicTransform for InsertCallNodes {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        let idxes = graph.nodes().collect::<Vec<_>>();
        for idx in idxes {
            let node = graph.get_node(idx);
            if FuncNode::downcastable(node) {
                let pred_idxes = graph.preds(idx).collect::<Vec<_>>();
                for pred_idx in pred_idxes {
                    // For every pred that is not a call node,
                    // insert a call node
                    let pred = graph.get_node(pred_idx);
                    match CallNode::concrete(pred) {
                        Some(_) => {}
                        None => {
                            self.result.did_work();
                            let call_node = graph.add_node(CallNode { args: vec![] });

                            let edge_type = graph.rmv_edge(pred_idx, idx);
                            graph.add_edge(pred_idx, call_node, edge_type);
                            graph.add_edge(call_node, idx, NoneEdge.into());
                        }
                    }
                }
            }
        }
        &self.result
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::{tests::*, BasicTransform};

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

        graph.write_dot("insert_call.dot");
    }
}
