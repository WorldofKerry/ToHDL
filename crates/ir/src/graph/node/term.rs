use crate::expr::*;

#[derive(Clone, PartialEq)]
pub struct TermNode {
    pub values: Vec<Expr>,
}

impl std::fmt::Display for TermNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.values)
    }
}
