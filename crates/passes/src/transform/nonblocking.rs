use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::*;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

#[derive(Default)]
pub struct Nonblocking {
    result: TransformResultType,
}

impl Transform for Nonblocking {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        self.start(graph, graph.get_entry(), &mut BTreeMap::new());
        &self.result
    }
}

impl Nonblocking {
    pub fn start(
        &mut self,
        graph: &mut CFG,
        idx: NodeIndex,
        mapping: &mut BTreeMap<VarExpr, Expr>,
    ) {
        let graph_copy = graph.clone();
        let node = &mut graph.get_node_mut(idx);
        println!("visiting {} {} with {:?}", idx, node, mapping);
        for value in node.referenced_exprs_mut() {
            value.backwards_replace(mapping);
        }
        if let Some(AssignNode { lvalue, rvalue }) = AssignNode::concrete(node) {
            mapping.insert(lvalue.clone(), rvalue.clone());
            for succ in graph.succs(idx).collect::<Vec<_>>() {
                self.start(graph, succ, &mut mapping.clone());
            }
        } else if CallNode::downcastable(node) {
            let call_succs = graph_copy.succs(idx).collect::<Vec<_>>();
            if call_succs.len() == 0 {
                return;
            }
            assert_eq!(call_succs.len(), 1);
            // Replace call-func pair with assign node
            let lvalues = FuncNode::concrete(graph_copy.get_node(call_succs[0]))
                .unwrap()
                .params
                .clone();
            let rvalues = CallNode::concrete(node).unwrap().args.clone();
            let mut assign_nodes = vec![];
            for (lvalue, rvalue) in lvalues.into_iter().zip(rvalues.into_iter()) {
                assign_nodes.push(AssignNode {
                    lvalue,
                    rvalue: Expr::Var(rvalue),
                });
            }
            let mut new_idx = idx;
            for assign_node in assign_nodes {
                let assign_idx = graph.insert_node(assign_node.clone(), idx, Edge::None);
                println!("assign_idx {} {}", assign_idx, assign_node);
                new_idx = assign_idx;
            }
            graph.rmv_node_and_reattach(call_succs[0]);
            graph.rmv_node_and_reattach(idx);
            for succ in graph.succs(new_idx).collect::<Vec<_>>() {
                self.start(graph, succ, &mut mapping.clone());
            }
        } else {
            for succ in graph.succs(idx).collect::<Vec<_>>() {
                self.start(graph, succ, &mut mapping.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimize::RemoveUnreadVars;
    use crate::tests::*;
    use crate::transform::*;

    //     pub fn make_even_fib() -> graph::CFG {
    //         let code = r#"
    // def even_fib():
    //     a = 0
    //     b = a
    //     yield b
    // "#;
    //         let visitor = tohdl_frontend::AstVisitor::from_text(code);

    //         let graph = visitor.get_graph();

    //         graph
    //     }

    #[test]
    fn odd_fib() {
        let mut graph = make_even_fib();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);
        insert_phi::InsertPhi::default().apply(&mut graph);
        make_ssa::MakeSSA::default().apply(&mut graph);

        let mut lower = LowerToFsm::default();
        lower.apply(&mut graph);

        write_graph(&graph, "lower_to_fsm.dot");

        // Write all new subgraphs to files
        for (i, subgraph) in lower.subgraphs.iter().enumerate() {
            let mut subgraph = subgraph.clone();
            Nonblocking::transform(&mut subgraph);
            RemoveUnreadVars::transform(&mut subgraph);
            write_graph(&subgraph, format!("nonblocking_{}.dot", i).as_str());
        }
    }
}
