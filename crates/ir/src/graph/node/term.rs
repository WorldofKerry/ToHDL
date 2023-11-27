use crate::expr::*;

use super::DataFlow;

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

impl DataFlow for TermNode {
    fn referenced_vars(&self) -> Vec<&VarExpr> {
        self.values.iter().flat_map(|v| v.get_vars_iter()).collect()
    }
    fn reference_vars_mut(&mut self) -> Vec<&mut VarExpr> {
        self.values
            .iter_mut()
            .flat_map(|v| v.get_vars_iter_mut())
            .collect()
    }
    fn read_exprs_mut(&mut self) -> Vec<&mut Expr> {
        let mut ret = vec![];
        for value in &mut self.values {
            ret.push(value);
        }
        ret
    }
}
