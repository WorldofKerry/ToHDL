use std::any::Any;

use crate::expr::*;

use super::{NodeLike, ReadsVariables, WroteVariables};

#[derive(Clone, PartialEq, Debug)]
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

impl WroteVariables for BranchNode {}

impl NodeLike for BranchNode {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
