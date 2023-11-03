use crate::ir::expr::*;

#[derive(Clone)]
pub struct AssignNode {
    pub lvalue: VarExpr,
    pub rvalue: Expr,
}

impl std::fmt::Display for AssignNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} = {}", self.lvalue, self.rvalue)
    }
}
