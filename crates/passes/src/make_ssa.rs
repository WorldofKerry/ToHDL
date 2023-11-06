use std::collections::{BTreeMap, BTreeSet};

use super::*;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

pub struct MakeSSA {
    visited: BTreeSet<usize>,
    var_counter: BTreeMap<VarExpr, usize>,
    stacks: BTreeMap<VarExpr, Vec<usize>>,
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

    /// Gets block of statements
    pub(crate) fn block(&self, graph: &DiGraph, node: usize) -> Vec<usize> {
        return graph.dfs(node, &|n| match n {
            Node::Call(_) => false,
            _ => true,
        });
    }

    /// Generate new name
    pub(crate) fn gen_name(&mut self, var: &VarExpr) -> VarExpr {
        println!("gen_name before {:?}", self.stacks);

        let count = *self.var_counter.get(&var).unwrap_or(&0);
        self.var_counter.insert(var.clone(), count + 1);

        let name = format!("{}.{}", var.name, count);
        let new_var = VarExpr::new(&name);

        // Update var stack
        if let Some(stack) = self.stacks.get_mut(&var) {
            stack.push(count + 1);
        } else {
            self.stacks.insert(var.clone(), vec![count + 1]);
        }

        // Update var mapping
        self.var_mapping.insert(new_var.clone(), var.clone());

        println!("gen_name after {:?}", self.stacks);
        new_var
    }

    /// Update LHS and RHS
    fn update_lhs_rhs(&mut self, stmt: Node) {
        match stmt {
            Node::Assign(AssignNode { lvalue, rvalue }) => {
                let new_lvalue = self.gen_name(&lvalue);
                let new_rvalue = match rvalue {
                    Expr::Var(var) => Expr::Var(self.var_mapping.get(&var).unwrap().clone()),
                    _ => rvalue,
                };
            }
            _ => {}
        }
    }
}

impl Transform for MakeSSA {
    fn transform(&mut self, graph: &mut DiGraph) {
        for node in graph.nodes() {
            println!("node {}", node);
            match graph.get_node_mut(node) {
                Node::Assign(assign) => {
                    assign.lvalue = self.gen_name(&assign.lvalue);
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

        assert_eq!(MakeSSA::new().block(&graph, 5), vec![5, 1, 2, 3, 4]);

        let result = MakeSSA::new().transform(&mut graph);

        println!("result {:?}", result);

        write_graph(&graph, "make_ssa.dot");
    }
}
