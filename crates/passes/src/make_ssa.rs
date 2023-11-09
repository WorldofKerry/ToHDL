use std::collections::{BTreeMap, BTreeSet};

use super::*;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

pub struct MakeSSA {
    visited: BTreeSet<usize>,
    var_counter: BTreeMap<VarExpr, usize>,
    stacks: BTreeMap<VarExpr, Vec<VarExpr>>,
    var_mapping: BTreeMap<VarExpr, VarExpr>,
    separater: &'static str,
}

impl Transform for MakeSSA {
    /// Applies transformation
    fn transform(&mut self, graph: &mut DiGraph) {
        self.rename(graph, 0)
    }
}

impl MakeSSA {
    pub fn new() -> Self {
        Self {
            visited: BTreeSet::new(),
            var_counter: BTreeMap::new(),
            stacks: BTreeMap::new(),
            var_mapping: BTreeMap::new(),
            separater: ".",
        }
    }

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
        for node in graph.dfs(0) {
            match graph.get_node_mut(node) {
                Node::Assign(AssignNode { lvalue, rvalue }) => {
                    *lvalue =
                        VarExpr::new(&lvalue.name.split(self.separater).collect::<Vec<_>>()[0]);
                    *rvalue = rvalue.backwards_replace(&self.make_revert_mapping(rvalue));
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
                        *value = value.backwards_replace(&self.make_revert_mapping(value));
                    }
                }
                Node::Branch(BranchNode { cond }) => {
                    *cond = cond.backwards_replace(&self.make_revert_mapping(cond));
                }
            }
        }
    }

    /// Get nodes within call block
    pub(crate) fn nodes_in_call_block(&self, graph: &DiGraph, node: usize) -> Vec<usize> {
        return graph.descendants_internal(node, &|n| match n {
            Node::Call(_) => false,
            _ => true,
        });
    }

    /// Gets descendant call nodes
    pub(crate) fn call_descendants(&self, graph: &DiGraph, node: usize) -> Vec<usize> {
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
                let new_rvalue = rvalue.backwards_replace(&self.make_mapping());
                let new_lvalue = self.gen_name(&lvalue);

                *rvalue = new_rvalue;
                *lvalue = new_lvalue;
            }
            Node::Branch(BranchNode { cond }) => {
                let new_cond = cond.backwards_replace(&self.make_mapping());
                *cond = new_cond;
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

    fn rename(&mut self, graph: &mut DiGraph, node: usize) {
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
                        let wrapped =
                            Expr::Var(param.clone()).backwards_replace(&self.make_mapping());
                        *param = match wrapped {
                            Expr::Var(var) => var,
                            _ => panic!("wrapped is not var"),
                        };
                    }
                }
                _ => {
                    panic!("descendant is not call node")
                }
            }
        }

        // DFS on dominator tree
        let dominance = petgraph::algo::dominators::simple_fast(&graph.0, 0.into());
        for s in graph.dfs(0) {
            // if node dominates s
            let dominates_s = dominance
                .dominators(petgraph::graph::NodeIndex::new(s))
                .unwrap()
                .collect::<Vec<petgraph::graph::NodeIndex>>();
            if dominates_s.contains(&petgraph::graph::NodeIndex::new(node)) {
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
    use super::super::tests::*;
    use super::*;

    #[test]
    fn range() {
        let mut graph = make_range();

        insert_func::InsertFuncNodes {}.transform(&mut graph);
        insert_call::InsertCallNodes {}.transform(&mut graph);
        insert_phi::InsertPhi {}.transform(&mut graph);

        assert_eq!(MakeSSA::new().nodes_in_call_block(&graph, 7), vec![7, 3, 4]);

        assert_eq!(MakeSSA::new().call_descendants(&graph, 7), vec![10]);

        let result = MakeSSA::new().transform(&mut graph);

        println!("result {:?}", result);

        write_graph(&graph, "make_ssa.dot");
    }

    #[test]
    fn fib() {
        let mut graph = make_fib();

        insert_func::InsertFuncNodes {}.transform(&mut graph);
        insert_call::InsertCallNodes {}.transform(&mut graph);
        insert_phi::InsertPhi {}.transform(&mut graph);

        // assert_eq!(
        //     MakeSSA::new().nodes_in_call_block(&graph, 5),
        //     vec![5, 1, 2, 3, 4]
        // );

        // assert_eq!(MakeSSA::new().call_descendants(&graph, 5), vec![7]);

        MakeSSA::new().transform(&mut graph);
        MakeSSA::new().revert_ssa_dangerous(&mut graph);
        MakeSSA::new().transform(&mut graph);
        MakeSSA::new().transform(&mut graph);
        MakeSSA::new().revert_ssa_dangerous(&mut graph);
        MakeSSA::new().transform(&mut graph);

        write_graph(&graph, "make_ssa.dot");
    }

    #[test]
    fn branch() {
        let mut graph = make_branch();

        insert_func::InsertFuncNodes {}.transform(&mut graph);
        insert_call::InsertCallNodes {}.transform(&mut graph);
        insert_phi::InsertPhi {}.transform(&mut graph);

        // assert_eq!(
        //     MakeSSA::new().nodes_in_call_block(&graph, 5),
        //     vec![5, 1, 2, 3, 4]
        // );

        // assert_eq!(MakeSSA::new().call_descendants(&graph, 5), vec![7]);

        MakeSSA::new().transform(&mut graph);

        write_graph(&graph, "make_ssa.dot");
    }
}
