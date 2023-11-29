use crate::expr::*;

use super::DataFlow;

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
    fn referenced_vars(&self) -> Vec<&VarExpr> {
        self.rvalue.get_vars_iter().collect()
    }
    fn defined_vars(&self) -> Vec<&VarExpr> {
        vec![&self.lvalue]
    }
    fn referenced_vars_mut(&mut self) -> Vec<&mut VarExpr> {
        self.rvalue.get_vars_iter_mut().collect()
    }
    fn defined_vars_mut(&mut self) -> Vec<&mut VarExpr> {
        vec![&mut self.lvalue]
    }
    fn referenced_exprs_mut(&mut self) -> Vec<&mut Expr> {
        vec![&mut self.rvalue]
    }
    fn undefine_var(&mut self, _var: &VarExpr) -> bool {
        true
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
