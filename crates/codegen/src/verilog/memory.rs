//! Transforms the graph to use memory / flip flops
//! Replaces the root func node with loads from memory
//! Replaces the leaf call nodes with stores to memory

use tohdl_ir::expr::*;

use tohdl_ir::graph::*;
use tohdl_passes::Transform;
use tohdl_passes::TransformResultType;

/// Special assignment that cannot be removed
#[derive(Clone, PartialEq, Debug)]
pub struct MemoryNode {
    pub lvalue: VarExpr,
    pub rvalue: Expr,
}

impl std::fmt::Display for MemoryNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Memory ({} = {})", self.lvalue, self.rvalue)
    }
}

impl DataFlow for MemoryNode {
    fn referenced_vars(&self) -> Vec<&VarExpr> {
        self.rvalue.get_vars_iter().collect()
    }
    fn declared_vars(&self) -> Vec<&VarExpr> {
        vec![&self.lvalue]
    }
    fn referenced_vars_mut(&mut self) -> Vec<&mut VarExpr> {
        self.rvalue.get_vars_iter_mut().collect()
    }
    fn declared_vars_mut(&mut self) -> Vec<&mut VarExpr> {
        vec![&mut self.lvalue]
    }
    fn referenced_exprs_mut(&mut self) -> Vec<&mut Expr> {
        vec![&mut self.rvalue]
    }
    fn undefine_var(&mut self, _var: &VarExpr) -> bool {
        false
    }
    fn defined_vars(&self) -> std::collections::BTreeMap<&VarExpr, &Expr> {
        [(&self.lvalue, &self.rvalue)].into()
    }
}

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
            let preds = graph.preds(idx).collect::<Vec<_>>();
            let succs = graph.succs(idx).collect::<Vec<_>>();

            // Only root or leaves should use memory, otherwise an assign is ok
            // This allows optimizer to optimize the assign nodes more
            let use_mem = preds.is_empty() || succs.is_empty();

            let node = graph.get_node(idx).clone();
            if let Some(FuncNode { params }) = FuncNode::concrete(&node) {
                for (i, param) in params.iter().enumerate() {
                    if use_mem {
                        graph.insert_node(
                            MemoryNode {
                                lvalue: param.clone(),
                                rvalue: Expr::Var(VarExpr::new(&format!("mem_{}", i))),
                            },
                            idx,
                            Edge::None,
                        );
                    } else {
                        graph.insert_node(
                            AssignNode {
                                lvalue: param.clone(),
                                rvalue: Expr::Var(VarExpr::new(&format!("mem_{}", i))),
                            },
                            idx,
                            Edge::None,
                        );
                    }
                }
            } else if let Some(CallNode { args }) = CallNode::concrete(&node) {
                for (i, arg) in args.iter().enumerate() {
                    if use_mem {
                        graph.insert_node(
                            MemoryNode {
                                lvalue: VarExpr::new(&format!("mem_{}", i)),
                                rvalue: Expr::Var(arg.clone()),
                            },
                            idx,
                            Edge::None,
                        );
                    } else {
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
