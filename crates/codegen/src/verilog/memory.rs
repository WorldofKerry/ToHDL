//! Transforms the graph to use memory / flip flops
//! Replaces the root func node with loads from memory
//! Replaces the leaf call nodes with stores to memory

use tohdl_ir::expr::*;

use tohdl_ir::graph::*;
use tohdl_passes::Transform;
use tohdl_passes::TransformResultType;

/// Special assignment that cannot be removed
#[derive(Clone, PartialEq, Debug)]
pub struct LoadNode {
    pub lvalue: VarExpr,
    pub rvalue: Expr,
}

impl std::fmt::Display for LoadNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Load {} = {}", self.lvalue, self.rvalue)
    }
}

impl DataFlow for LoadNode {
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

/// Special assignment that cannot be removed
#[derive(Clone, PartialEq, Debug)]
pub struct StoreNode {
    pub lvalue: VarExpr,
    pub rvalue: Expr,
}

impl std::fmt::Display for StoreNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Store {} = {}", self.lvalue, self.rvalue)
    }
}

impl DataFlow for StoreNode {
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

/// State transition node
#[derive(Clone, PartialEq, Debug)]
pub struct NextStateNode {}

impl std::fmt::Display for NextStateNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Nextstate")
    }
}

impl DataFlow for NextStateNode {}

/// Root and leaf func and call nodes do not make sense in context of Verilog
/// This pass replaces them with load and store nodes to registers
#[derive(Default)]
pub struct UseMemory {
    result: TransformResultType,
    max_memory: usize,
}

impl Transform for UseMemory {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        self.make_func_and_calls_use_mem(graph);
        &self.result
    }
}

impl UseMemory {
    pub fn max_memory(&self) -> usize {
        self.max_memory
    }

    pub(crate) fn make_func_and_calls_use_mem(&mut self, graph: &mut CFG) {
        for idx in graph.nodes().collect::<Vec<_>>() {
            let preds = graph.preds(idx).collect::<Vec<_>>();
            let succs = graph.succs(idx).collect::<Vec<_>>();

            // Only root or leaves should use memory, otherwise an assign is ok
            // This allows optimizer to optimize the assign nodes more
            let use_mem = preds.is_empty() || succs.is_empty();

            let node = graph.get_node(idx).clone();
            if let Some(FuncNode { params }) = FuncNode::concrete(&node) {
                self.max_memory = std::cmp::max(self.max_memory, params.len());
                for (i, param) in params.iter().enumerate() {
                    if use_mem {
                        graph.insert_node_after(
                            LoadNode {
                                lvalue: param.clone(),
                                rvalue: Expr::Var(
                                    VarExpr::builder()
                                        .name(&format!("mem_{}", i))
                                        .type_(VarType::Pointer(Box::new(VarType::Int)))
                                        .build(),
                                ),
                            },
                            idx,
                            NoneEdge.into()
                        );
                    } else {
                        // graph.insert_node_after(
                        //     AssignNode {
                        //         lvalue: param.clone(),
                        //         rvalue: Expr::Var(
                        //             VarExpr::builder()
                        //                 .name(&format!("mem_{}", i))
                        //                 .type_(VarType::Pointer(Box::new(VarType::Int)))
                        //                 .build(),
                        //         ),
                        //     },
                        //     idx,
                        //     Edge::None,
                        // );
                    }
                }
            } else if let Some(CallNode { args }) = CallNode::concrete(&node) {
                self.max_memory = std::cmp::max(self.max_memory, args.len());
                for (i, arg) in args.iter().enumerate() {
                    if use_mem {
                        graph.insert_node_before(
                            StoreNode {
                                lvalue: VarExpr::builder()
                                    .name(&format!("mem_{}", i))
                                    .type_(VarType::Pointer(Box::new(VarType::Int)))
                                    .build(),
                                rvalue: Expr::Var(arg.clone()),
                            },
                            idx,
                            NoneEdge.into(),
                        );
                    } else {
                        // graph.insert_node_before(
                        //     AssignNode {
                        //         lvalue: VarExpr::builder()
                        //             .name(&format!("mem_{}", i))
                        //             .type_(VarType::Pointer(Box::new(VarType::Int)))
                        //             .build(),
                        //         rvalue: Expr::Var(arg.clone()),
                        //     },
                        //     idx,
                        //     Edge::None,
                        // );
                    }
                }
            }
        }
        for idx in graph.nodes().collect::<Vec<_>>() {
            let node = graph.get_node(idx).clone();
            let preds = graph.preds(idx).collect::<Vec<_>>();
            let succs = graph.succs(idx).collect::<Vec<_>>();
            let use_mem = preds.is_empty() || succs.is_empty();

            if FuncNode::downcastable(&node) && use_mem {
                graph.rmv_node_and_reattach(idx);
            } else if CallNode::downcastable(&node) && use_mem {
                // panic!();
                // graph.rmv_node_and_reattach(idx);
                graph.replace_node(idx, NextStateNode {});
            } else if CallNode::downcastable(&node) && !use_mem {
                // graph.rmv_node_and_reattach(idx);
                // graph.replace_node(idx, NextStateNode {});
            }
        }
    }
}

/// Within a single state, loading from memory does not make sense,
/// as all assignments are non-blocking,
/// so the load dst cannot be used within that state.
/// This pass removes all load nodes.
/// Should be ran after the non-blocking pass is ran
/// Additionally, this pass removes all assigns to memory nodes
#[derive(Default)]
pub struct RemoveLoadsEtc {
    result: TransformResultType,
}

impl Transform for RemoveLoadsEtc {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        let mut to_be_removed = vec![];
        for idx in graph.nodes().collect::<Vec<_>>() {
            let node = graph.get_node(idx);
            if LoadNode::downcastable(node) {
                to_be_removed.push(idx);
            }
            if let Some(AssignNode { lvalue, rvalue: _ }) = AssignNode::concrete(node) {
                match lvalue.type_ {
                    VarType::Pointer(_) => to_be_removed.push(idx),
                    _ => {}
                }
            }
        }
        for idx in to_be_removed {
            graph.rmv_node_and_reattach(idx);
        }
        &self.result
    }
}
