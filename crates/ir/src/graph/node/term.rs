use crate::expr::*;

use super::ReadsVariables;

#[derive(Clone, PartialEq)]
pub struct TermNode {
    pub values: Vec<Expr>,
}

impl std::fmt::Display for TermNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let buf = self
            .values
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "({})", buf)
    }
}

impl ReadsVariables for TermNode {
    fn read_vars(&mut self) -> Vec<&mut VarExpr> {
        self.values
            .iter_mut()
            .flat_map(|v| v.get_vars_iter())
            .collect()
    }
}
