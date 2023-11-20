use std::any::Any;

use crate::expr::*;

use super::{DataFlow, NodeLike};

#[derive(Clone, PartialEq, Debug)]
pub struct BranchNode {
    pub cond: Expr,
}

impl std::fmt::Display for BranchNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "if {}", self.cond)
    }
}

impl DataFlow for BranchNode {
    fn read_vars(&mut self) -> Vec<&mut VarExpr> {
        self.cond.get_vars_iter().collect()
    }
}
