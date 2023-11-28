//! Transforms the graph to use memory / flip flops
//! Replaces the root func node with loads from memory
//! Replaces the leaf call nodes with stores to memory

use std::collections::BTreeMap;

use crate::*;
use tohdl_ir::expr::*;
use tohdl_ir::graph::DataFlow;
use tohdl_ir::graph::*;
use tohdl_passes::Transform;
use tohdl_passes::TransformResultType;

#[derive(Default)]
pub struct UseMemory {
    result: TransformResultType,
}

impl Transform for UseMemory {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        self.make_func_and_calls_use_mem(graph);
        &self.result
    }
}

impl UseMemory {
    pub(crate) fn make_func_and_calls_use_mem(&mut self, graph: &mut CFG) {
        for idx in graph.nodes().collect::<Vec<_>>() {
            let node = graph.get_node(idx).clone();
            if let Some(FuncNode { params }) = FuncNode::concrete(&node) {
                for (i, param) in params.iter().enumerate() {
                    graph.insert_node(
                        AssignNode {
                            lvalue: param.clone(),
                            rvalue: Expr::Var(VarExpr::new(&format!("mem_{}", i))),
                        },
                        idx,
                        Edge::None,
                    );
                }
            } else if let Some(CallNode { args }) = CallNode::concrete(&node) {
                for (i, arg) in args.iter().enumerate() {
                    graph.insert_node(
                        AssignNode {
                            lvalue: VarExpr::new(&format!("mem_{}", i)),
                            rvalue: Expr::Var(arg.clone()),
                        },
                        idx,
                        Edge::None,
                    );
                }
            }
        }
        for idx in graph.nodes().collect::<Vec<_>>() {
            let node = graph.get_node(idx).clone();
            if FuncNode::downcastable(&node) {
                graph.rmv_node_and_reattach(idx);
            } else if CallNode::downcastable(&node) {
                graph.rmv_node_and_reattach(idx);
            }
        }
    }
}
