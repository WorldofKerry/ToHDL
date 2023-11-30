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
    fn declared_vars(&self) -> Vec<&VarExpr> {
        // return vec![];
        self.params.iter().collect()
    }
    fn declared_vars_mut(&mut self) -> Vec<&mut VarExpr> {
        // return vec![];
        self.params.iter_mut().collect()
    }
    fn undefine_var(&mut self, var: &VarExpr) -> bool {
        let index = self.params.iter().position(|x| x == var).unwrap();
        self.params.remove(index);
        false
    }
}
