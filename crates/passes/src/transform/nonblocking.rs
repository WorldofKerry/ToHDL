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
        self.replace_call_and_func_nodes(graph, graph.get_entry());
        self.rmv_call_and_func_nodes(graph);
        self.start(graph, graph.get_entry(), &mut BTreeMap::new());
        &self.result
    }
}

impl Nonblocking {
    /// Replaces call and func nodes with assign nodes
    /// Note that graph must be a DAG to not have correctness errors
    pub(crate) fn replace_call_and_func_nodes(&mut self, graph: &mut CFG, idx: NodeIndex) {
        let graph_copy = graph.clone();
        let node = &mut graph.get_node_mut(idx);
        if CallNode::downcastable(node) {
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
            for assign_node in assign_nodes {
                graph.insert_node(assign_node.clone(), idx, Edge::None);
            }
        }
        for succ in graph.succs(idx).collect::<Vec<_>>() {
            self.replace_call_and_func_nodes(graph, succ);
        }
    }

    /// Excludes func nodes with no preds, and call nodes with no succs
    pub(crate) fn rmv_call_and_func_nodes(&mut self, graph: &mut CFG) {
        for idx in graph.nodes().collect::<Vec<_>>() {
            let node = graph.get_node(idx).clone();
            if FuncNode::downcastable(&node) && graph.preds(idx).collect::<Vec<_>>().len() != 0 {
                graph.rmv_node_and_reattach(idx);
            } else if CallNode::downcastable(&node)
                && graph.succs(idx).collect::<Vec<_>>().len() != 0
            {
                graph.rmv_node_and_reattach(idx);
            }
        }
    }

    pub fn start(
        &mut self,
        graph: &mut CFG,
        idx: NodeIndex,
        mapping: &mut BTreeMap<VarExpr, Expr>,
    ) {
        let node = &mut graph.get_node_mut(idx);
        println!("visiting {} {}", idx, node);
        for value in node.referenced_exprs_mut() {
            value.backwards_replace(mapping);
        }
        if let Some(AssignNode { lvalue, rvalue }) = AssignNode::concrete(node) {
            mapping.insert(lvalue.clone(), rvalue.clone());
            for succ in graph.succs(idx).collect::<Vec<_>>() {
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
            // RemoveUnreadVars::transform(&mut subgraph);
            write_graph(&subgraph, format!("nonblocking_{}.dot", i).as_str());
        }
    }
}
