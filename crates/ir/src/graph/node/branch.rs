use crate::expr::*;

use super::ReadsVariables;

#[derive(Clone, PartialEq)]
pub struct BranchNode {
    pub cond: Expr,
}

impl std::fmt::Display for BranchNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "if {}", self.cond)
    }
}

impl ReadsVariables for BranchNode {
    fn read_vars(&mut self) -> Vec<&mut VarExpr> {
        self.cond.get_vars_iter().collect()
    }
}