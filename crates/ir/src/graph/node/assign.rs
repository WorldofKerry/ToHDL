use std::any::Any;

use crate::expr::*;

use super::{DataFlow, NodeLike};

#[derive(Clone, PartialEq, Debug)]
pub struct AssignNode {
    pub lvalue: VarExpr,
    pub rvalue: Expr,
}

impl std::fmt::Display for AssignNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} = {}", self.lvalue, self.rvalue)
    }
}

impl DataFlow for AssignNode {
    fn read_vars(&self) -> Vec<&VarExpr> {
        self.rvalue.get_vars()
    }
    fn wrote_vars(&self) -> Vec<&VarExpr> {
        vec![&self.lvalue]
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_modifying_read_vars() {
        // let mut node = AssignNode {
        //     lvalue: VarExpr::new("x"),
        //     rvalue: Expr::Var(VarExpr::new("y")),
        // };
        // assert_eq!(node.read_vars(), vec![VarExpr::new("y")]);
        // node.rvalue = Expr::Var(VarExpr::new("z"));
        // assert_eq!(node.read_vars(), vec![VarExpr::new("z")]);
    }
}
