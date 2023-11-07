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
        {
            let mut binding = self.stacks.borrow_mut();
            let stack = binding.entry(var.clone()).or_default();
            stack.push(new_var.clone());
        }

        // Update var mapping
        self.var_mapping
            .borrow_mut()
            .insert(new_var.clone(), var.clone());

        println!("gen_name after {:?}", self.stacks.borrow_mut());
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
}

impl Default for MakeSSA {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform for MakeSSA {
    fn transform(&self, graph: &mut DiGraph) {
        for stmt in self.nodes_in_call_block(graph, 0) {
            self.update_lhs_rhs(graph.get_node_mut(stmt));
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
