use crate::expr::VarExpr;

use super::DataFlow;

#[derive(Clone, PartialEq)]
pub struct FuncNode {
    pub params: Vec<VarExpr>,
}

impl std::fmt::Display for FuncNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let args = self
            .params
            .iter()
            .map(|a| format!("{}", a))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "func({})", args)
    }
}

impl DataFlow for FuncNode {
    fn wrote_vars(&self) -> Vec<&VarExpr> {
        self.params.iter().collect()
    }
}
