use crate::expr::*;

use super::{ReadsVariables, WroteVariables};

#[derive(Clone, PartialEq)]
pub struct AssignNode {
    pub lvalue: VarExpr,
    pub rvalue: Expr,
}

impl std::fmt::Display for AssignNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} = {}", self.lvalue, self.rvalue)
    }
}

impl ReadsVariables for AssignNode {
    fn read_vars(&mut self) -> Vec<&mut VarExpr> {
        self.rvalue.get_vars_iter().collect()
    }
}

impl WroteVariables for AssignNode {
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
