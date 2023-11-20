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
    fn read_vars(&self) -> Vec<&VarExpr> {
        self.cond.get_vars()
    }
}
