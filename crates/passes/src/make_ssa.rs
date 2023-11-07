use std::collections::{BTreeMap, BTreeSet};

use super::*;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

pub struct MakeSSA {
    visited: BTreeSet<usize>,
    var_counter: BTreeMap<VarExpr, usize>,
    stacks: BTreeMap<VarExpr, Vec<VarExpr>>,
    var_mapping: BTreeMap<VarExpr, VarExpr>,
    global_vars: BTreeSet<VarExpr>,
}

impl MakeSSA {
    pub fn new() -> Self {
        Self {
            visited: BTreeSet::new(),
            var_counter: BTreeMap::new(),
            stacks: BTreeMap::new(),
            var_mapping: BTreeMap::new(),
            global_vars: BTreeSet::new(),
        }
    }

    /// Get subtree excluding and stopping at call nodes
    pub(crate) fn subtree_excluding(&self, graph: &DiGraph, node: usize) -> Vec<usize> {
        return graph.descendants_internal(node, &|n| match n {
            Node::Call(_) => false,
            _ => true,
        });
    }

    /// Get leaves of subtree stopping at call nodes
    pub(crate) fn subtree_leaves(&self, graph: &DiGraph, node: usize) -> Vec<usize> {
        return graph.descendants_leaves(node, &|n| match n {
            Node::Call(_) => true,
            _ => false,
        });
    }

    /// Geerate new name
    pub(crate) fn gen_name(&mut self, var: &VarExpr) -> VarExpr {
        println!("gen_name before {:?}", self.stacks);

        let count = *self.var_counter.get(&var).unwrap_or(&0);
        self.var_counter.insert(var.clone(), count + 1);

        let name = format!("{}.{}", var.name, count);
        let new_var = VarExpr::new(&name);

        // Update var stack
        if let Some(stack) = self.stacks.get_mut(&var) {
            stack.push(new_var.clone());
        } else {
            self.stacks.insert(var.clone(), vec![new_var.clone()]);
        }

        // Update var mapping
        self.var_mapping.insert(new_var.clone(), var.clone());

        println!("gen_name after {:?}", self.stacks);
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
            _ => {}
        }
    }

    /// Converts variable stack to mapping, by taking last element
    fn make_mapping(&self) -> BTreeMap<VarExpr, Expr> {
        self.stacks
            .iter()
            .map(|(var, stack)| (var.clone(), Expr::Var(stack.last().unwrap().clone())))
            .collect()
    }
}

impl Transform for MakeSSA {
    fn transform(&mut self, graph: &mut DiGraph) {
        for stmt in self.subtree_excluding(graph, 0) {
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

        assert_eq!(
            MakeSSA::new().subtree_excluding(&graph, 5),
            vec![5, 1, 2, 3, 4]
        );

        let result = MakeSSA::new().transform(&mut graph);

        println!("result {:?}", result);

        write_graph(&graph, "make_ssa.dot");
    }
}
