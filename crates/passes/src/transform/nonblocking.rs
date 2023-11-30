//! Makes graph useable in a fully nonblocking assignment context

use crate::*;
use std::collections::BTreeMap;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

#[derive(Default)]
pub struct Nonblocking {
    result: TransformResultType,
}

impl Transform for Nonblocking {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        Nonblocking::replace_call_and_func_nodes(graph, graph.get_entry());
        Nonblocking::rmv_call_and_func_nodes(graph);
        Nonblocking::recurse(graph, graph.get_entry(), &mut BTreeMap::new());
        &self.result
    }
}

impl Nonblocking {
    /// Replaces call and func nodes with assign nodes
    /// Note that graph must be a DAG to not have correctness errors
    pub(crate) fn replace_call_and_func_nodes(graph: &mut CFG, idx: NodeIndex) {
        let graph_copy = graph.clone();
        let node = &mut graph.get_node_mut(idx);
        if CallNode::downcastable(node) {
            let call_succs = graph_copy.succs(idx).collect::<Vec<_>>();
            if call_succs.is_empty() {
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
                graph.insert_node(assign_node, idx, Edge::None);
            }
        }
        for succ in graph.succs(idx).collect::<Vec<_>>() {
            Nonblocking::replace_call_and_func_nodes(graph, succ);
        }
    }

    /// Excludes func nodes with no preds, and call nodes with no succs
    pub(crate) fn included(idx: NodeIndex, node: &Box<dyn NodeLike>, graph: &CFG) -> bool {
        (FuncNode::downcastable(&node) && !graph.preds(idx).collect::<Vec<_>>().is_empty())
            || (CallNode::downcastable(&node) && !graph.succs(idx).collect::<Vec<_>>().is_empty())
    }

    /// Excludes func nodes with no preds, and call nodes with no succs
    pub(crate) fn rmv_call_and_func_nodes(graph: &mut CFG) {
        for idx in graph.nodes().collect::<Vec<_>>() {
            let node = graph.get_node(idx).clone();
            if Nonblocking::included(idx, &node, graph) {
                graph.rmv_node_and_reattach(idx);
            }
        }
    }

    /// Excludes func nodes with no preds, and call nodes with no succs
    pub fn recurse(graph: &mut CFG, idx: NodeIndex, mapping: &mut BTreeMap<VarExpr, Expr>) {
        // let included = Nonblocking::included(idx, &graph.get_node_mut(idx).clone(), graph);
        let node = &mut graph.get_node_mut(idx);
        println!("visiting {} {}", idx, node);
        for value in node.referenced_exprs_mut() {
            value.backwards_replace(mapping);
        }
        for (lhs, rhs) in node.defined_vars() {
            mapping.insert(lhs.clone(), rhs.clone());
        }
        for succ in graph.succs(idx).collect::<Vec<_>>() {
            Nonblocking::recurse(graph, succ, &mut mapping.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
