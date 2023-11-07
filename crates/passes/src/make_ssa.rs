use std::cell::RefCell;
use std::collections::{btree_map, BTreeMap, BTreeSet};

use super::*;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

pub struct MakeSSA {
    visited: RefCell<BTreeSet<usize>>,
    var_counter: RefCell<BTreeMap<VarExpr, usize>>,
    stacks: RefCell<BTreeMap<VarExpr, Vec<VarExpr>>>,
    var_mapping: RefCell<BTreeMap<VarExpr, VarExpr>>,
    global_vars: RefCell<BTreeSet<VarExpr>>,
}

impl Transform for MakeSSA {
    fn transform(&self, graph: &mut DiGraph) {
        self.rename(graph, 0)
    }
}

impl MakeSSA {
    pub fn new() -> Self {
        Self {
            visited: RefCell::new(BTreeSet::new()),
            var_counter: RefCell::new(BTreeMap::new()),
            stacks: RefCell::new(BTreeMap::new()),
            var_mapping: RefCell::new(BTreeMap::new()),
            global_vars: RefCell::new(BTreeSet::new()),
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

    /// Geerate new name
    pub(crate) fn gen_name(&self, var: &VarExpr) -> VarExpr {
        println!("gen_name before {:?}", self.stacks.borrow_mut());

        let count = *self.var_counter.borrow_mut().get(&var).unwrap_or(&0);
        self.var_counter.borrow_mut().insert(var.clone(), count + 1);

        let name = format!("{}.{}", var.name, count);
        let new_var = VarExpr::new(&name);

        // Update var stack
        let mut binding = self.stacks.borrow_mut();
        let stack = binding.entry(var.clone()).or_default();
        stack.push(new_var.clone());

        // Update var mapping
        self.var_mapping
            .borrow_mut()
            .insert(new_var.clone(), var.clone());

        println!("gen_name after {:?}", self.stacks);
        new_var
    }

    /// Update LHS and RHS
    fn update_lhs_rhs(&self, stmt: &mut Node) {
        match stmt {
            Node::Assign(AssignNode { lvalue, rvalue }) => {
                // Note that old mapping is used for rvalue
                let new_rvalue = rvalue.backwards_replace(&self.make_mapping());
                let new_lvalue = self.gen_name(&lvalue);

                *rvalue = new_rvalue;
                *lvalue = new_lvalue;
            }
            _ => {}
        }
    }

    /// Converts variable stack to mapping, by taking last element
    fn make_mapping(&self) -> BTreeMap<VarExpr, Expr> {
        self.stacks
            .borrow()
            .iter()
            .map(|(var, stack)| (var.clone(), Expr::Var(stack.last().unwrap().clone())))
            .collect()
    }

    fn rename(&self, graph: &mut DiGraph, node: usize) {
        // Check visited
        if self.visited.borrow().contains(&node) {
            return;
        }
        self.visited.borrow_mut().insert(node);

        println!("rename node {}", node);

        // For every stmt in call block, update lhs and rhs, creating new vars for ssa
        for stmt in self.nodes_in_call_block(graph, node) {
            self.update_lhs_rhs(graph.get_node_mut(stmt));
        }

        // For every desc call node, rename param to back of var stack
        for s in self.call_descendants(graph, node) {
            match graph.get_node_mut(s) {
                Node::Call(CallNode { ref mut params, .. }) => {
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
                    Node::Func(FuncNode { args: _, .. }) => {
                        self.rename(graph, s);
                    }
                    _ => {}
                }
            }
        }

        // Unwind stack
        match graph.get_node(node) {
            Node::Func(FuncNode { args }) => {
                for arg in args {
                    let mut binding = self.stacks.borrow_mut();
                    let stack = binding.entry(arg.clone()).or_default();
                    stack.pop();
                }
            }
            _ => {}
        }
        for stmt in self.nodes_in_call_block(graph, node) {
            match graph.get_node_mut(stmt) {
                Node::Assign(AssignNode { lvalue, .. }) => {
                    let mut binding = self.stacks.borrow_mut();
                    let stack = binding.entry(lvalue.clone()).or_default();
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
    fn main() {
        let mut graph = make_range();

        insert_func::InsertFuncNodes {}.transform(&mut graph);
        insert_call::InsertCallNodes {}.transform(&mut graph);
        insert_phi::InsertPhi {}.transform(&mut graph);

        assert_eq!(
            MakeSSA::new().nodes_in_call_block(&graph, 5),
            vec![5, 1, 2, 3, 4]
        );

        assert_eq!(MakeSSA::new().call_descendants(&graph, 5), vec![7]);

        let result = MakeSSA::new().transform(&mut graph);

        println!("result {:?}", result);

        write_graph(&graph, "make_ssa.dot");
    }
}
