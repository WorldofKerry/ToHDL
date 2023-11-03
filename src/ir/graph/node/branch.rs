use crate::ir::expr::*;

#[derive(Clone)]
pub struct BranchNode {
    pub cond: Expr,
}

impl std::fmt::Debug for BranchNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "if {:?}:", self.cond)
    }
}
