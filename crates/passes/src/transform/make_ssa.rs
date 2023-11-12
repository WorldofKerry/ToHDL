use std::collections::{BTreeMap, BTreeSet};

use crate::*;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

pub struct MakeSSA {
    visited: BTreeSet<NodeIndex>,
    var_counter: BTreeMap<VarExpr, usize>,
    stacks: BTreeMap<VarExpr, Vec<VarExpr>>,
    var_mapping: BTreeMap<VarExpr, VarExpr>,
    separater: &'static str,
    result: TransformResultType,
}

impl Default for MakeSSA {
    fn default() -> Self {
        Self {
            visited: BTreeSet::new(),
            var_counter: BTreeMap::new(),
            stacks: BTreeMap::new(),
            var_mapping: BTreeMap::new(),
            separater: ".",
            result: TransformResultType::default(),
        }
    }
}

impl Transform for MakeSSA {
    /// Applies transformation
    fn apply(&mut self, graph: &mut DiGraph) -> &TransformResultType {
        self.rename(graph, 0.into());
        &self.result
    }
}

impl MakeSSA {
    /// Make revert mapping
    fn make_revert_mapping(&self, expr: &Expr) -> BTreeMap<VarExpr, Expr> {
        let mut ret = BTreeMap::new();
        for var in expr.get_vars() {
            ret.insert(
                var.clone(),
                Expr::Var(VarExpr::new(
                    &var.name.split(self.separater).collect::<Vec<_>>()[0],
                )),
            );
        }
        ret
    }

    /// Revert SSA by removing separator from variable names
    /// Only retains correctness if reverted immediately after transforming to SSA
    pub fn revert_ssa_dangerous(&self, graph: &mut DiGraph) {
        for node in graph.dfs(0.into()) {
            match graph.get_node_mut(node) {
                Node::Assign(AssignNode { lvalue, rvalue }) => {
                    *lvalue =
                        VarExpr::new(&lvalue.name.split(self.separater).collect::<Vec<_>>()[0]);
                    rvalue.backwards_replace(&self.make_revert_mapping(rvalue));
                }
                Node::Func(FuncNode { params }) => {
                    for param in params {
                        *param =
                            VarExpr::new(&param.name.split(self.separater).collect::<Vec<_>>()[0]);
                    }
                }
                Node::Call(CallNode { args, .. }) => {
                    for arg in args {
                        *arg = VarExpr::new(&arg.name.split(self.separater).collect::<Vec<_>>()[0]);
                    }
                }
                Node::Return(TermNode { values }) | Node::Yield(TermNode { values }) => {
                    for value in values {
                        value.backwards_replace(&self.make_revert_mapping(value));
                    }
                }
                Node::Branch(BranchNode { cond }) => {
                    cond.backwards_replace(&self.make_revert_mapping(cond));
                }
            }
        }
    }

    /// Get nodes within call block
    pub(crate) fn nodes_in_call_block(&self, graph: &DiGraph, node: NodeIndex) -> Vec<NodeIndex> {
        return graph.descendants_internal(node, &|n| match n {
            Node::Call(_) => false,
            _ => true,
        });
    }

    /// Gets descendant call nodes
    pub(crate) fn call_descendants(&self, graph: &DiGraph, node: NodeIndex) -> Vec<NodeIndex> {
        return graph.descendants_leaves(node, &|n| match n {
            Node::Call(_) => true,
            _ => false,
        });
    }

    /// Generate new name
    pub(crate) fn gen_name(&mut self, var: &VarExpr) -> VarExpr {
        // println!("gen_name before {:?}", self.stacks);

        let count = *self.var_counter.get(&var).unwrap_or(&0);
        self.var_counter.insert(var.clone(), count + 1);

        let name = format!("{}.{}", var.name, count);
        let new_var = VarExpr::new(&name);

        // Update var stack
        let stack = self.stacks.entry(var.clone()).or_default();
        stack.push(new_var.clone());

        // Update var mapping
        self.var_mapping.insert(new_var.clone(), var.clone());

        // println!("gen_name after {:?}", self.stacks);
        new_var
    }

    /// Update LHS and RHS
    fn update_lhs_rhs(&mut self, stmt: &mut Node) {
        match stmt {
            Node::Assign(AssignNode { lvalue, rvalue }) => {
                // Note that old mapping is used for rvalue
                rvalue.backwards_replace(&self.make_mapping());
                *lvalue = self.gen_name(&lvalue);
            }
            Node::Branch(BranchNode { cond }) => {
                cond.backwards_replace(&self.make_mapping());
            }
            _ => {}
        }
    }

    /// Converts variable stack to mapping, by taking last element
    fn make_mapping(&self) -> BTreeMap<VarExpr, Expr> {
        self.stacks
            .iter()
            .map(|(var, stack)| {
                (
                    var.clone(),
                    Expr::Var(stack.last().unwrap_or(&VarExpr::new("unused")).clone()),
                )
            })
            .collect()
    }

    fn rename(&mut self, graph: &mut DiGraph, node: NodeIndex) {
        // Check visited
        if self.visited.contains(&node) {
            return;
        }
        self.visited.insert(node);

        println!("rename node {}", node);

        // Rename call params
        match graph.get_node_mut(node) {
            Node::Func(FuncNode { params }) => {
                for param in params {
                    *param = self.gen_name(param);
                }
            }
            _ => {}
        }
        // For every stmt in call block, update lhs and rhs, creating new vars for ssa
        for stmt in self.nodes_in_call_block(graph, node) {
            self.update_lhs_rhs(graph.get_node_mut(stmt));
        }

        // For every desc call node, rename param to back of var stack
        for s in self.call_descendants(graph, node) {
            match graph.get_node_mut(s) {
                Node::Call(CallNode {
                    args: ref mut params,
                    ..
                }) => {
                    for param in params {
                        if let Some(replacement) = self.var_mapping.get(param) {
                            *param = replacement.clone();
                        }
                    }
                }
                _ => {
                    panic!("descendant is not call node")
                }
            }
        }

        // DFS on dominator tree
        let dominance = petgraph::algo::dominators::simple_fast(&graph.0, 0.into());
        for s in graph.dfs(0.into()) {
            // if node dominates s
            let dominates_s = dominance
                .dominators(petgraph::graph::NodeIndex::new(s.into()))
                .unwrap()
                .collect::<Vec<petgraph::graph::NodeIndex>>();
            if dominates_s.contains(&petgraph::graph::NodeIndex::new(node.into())) {
                match graph.get_node(s) {
                    Node::Func(FuncNode { params: _, .. }) => {
                        self.rename(graph, s);
                    }
                    _ => {}
                }
            }
        }

        // Unwind stack
        match graph.get_node(node) {
            Node::Func(FuncNode { params: args }) => {
                for arg in args {
                    let stack = self.stacks.entry(arg.clone()).or_default();
                    stack.pop();
                }
            }
            _ => {}
        }
        for stmt in self.nodes_in_call_block(graph, node) {
            match graph.get_node_mut(stmt) {
                Node::Assign(AssignNode { lvalue, .. }) => {
                    let stack = self.stacks.entry(lvalue.clone()).or_default();
                    stack.pop();
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;
    use crate::transform::*;

    #[test]
    fn range() {
        let mut graph = make_range();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);
        insert_phi::InsertPhi::default().apply(&mut graph);

        assert_eq!(
            MakeSSA::default().nodes_in_call_block(&graph, 7.into()),
            vec![7, 3, 4]
                .iter()
                .map(|x| (*x).into())
                .collect::<Vec<_>>()
        );

        assert_eq!(MakeSSA::default().call_descendants(&graph, 7.into()), vec![10.into()]);

        MakeSSA::default().apply(&mut graph);

        write_graph(&graph, "make_ssa.dot");
    }

    #[test]
    fn fib() {
        let mut graph = make_fib();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);
        insert_phi::InsertPhi::default().apply(&mut graph);

        // assert_eq!(
        //     MakeSSA::new().nodes_in_call_block(&graph, 5),
        //     vec![5, 1, 2, 3, 4]
        // );

        // assert_eq!(MakeSSA::new().call_descendants(&graph, 5), vec![7]);

        MakeSSA::default().apply(&mut graph);
        MakeSSA::default().revert_ssa_dangerous(&mut graph);
        MakeSSA::default().apply(&mut graph);
        MakeSSA::default().apply(&mut graph);
        MakeSSA::default().revert_ssa_dangerous(&mut graph);
        MakeSSA::default().apply(&mut graph);

        write_graph(&graph, "make_ssa.dot");
    }

    #[test]
    fn branch() {
        let mut graph = make_branch();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);
        insert_phi::InsertPhi::default().apply(&mut graph);

        // assert_eq!(
        //     MakeSSA::new().nodes_in_call_block(&graph, 5),
        //     vec![5, 1, 2, 3, 4]
        // );

        // assert_eq!(MakeSSA::new().call_descendants(&graph, 5), vec![7]);

        MakeSSA::default().apply(&mut graph);

        write_graph(&graph, "make_ssa.dot");
    }
}