use super::{DataFlow, NodeLike};
use crate::expr::*;

pub trait MultiExpr {
    fn values(&self) -> &Vec<Expr>;
    fn values_mut(&mut self) -> &mut Vec<Expr>;
    fn name() -> &'static str;
    fn to_string(&self) -> String {
        let buf = self
            .values()
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{} ({})", Self::name(), buf)
    }
}

impl<T: MultiExpr + Clone> DataFlow for T {
    fn referenced_vars(&self) -> Vec<&VarExpr> {
        self.values()
            .iter()
            .flat_map(|v| v.get_vars_iter())
            .collect()
    }
    fn referenced_vars_mut(&mut self) -> Vec<&mut VarExpr> {
        self.values_mut()
            .iter_mut()
            .flat_map(|v| v.get_vars_iter_mut())
            .collect()
    }
    fn referenced_exprs_mut(&mut self) -> Vec<&mut Expr> {
        let mut ret = vec![];
        for value in self.values_mut() {
            ret.push(value);
        }
        ret
    }
}

#[derive(Clone, PartialEq)]
pub struct ReturnNode {
    pub values: Vec<Expr>,
}

impl MultiExpr for ReturnNode {
    fn values(&self) -> &Vec<Expr> {
        &self.values
    }

    fn values_mut(&mut self) -> &mut Vec<Expr> {
        &mut self.values
    }

    fn name() -> &'static str {
        "return"
    }
}

impl std::fmt::Display for ReturnNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", MultiExpr::to_string(self))
    }
}

#[derive(Clone, PartialEq)]
pub struct YieldNode {
    pub values: Vec<Expr>,
}

impl MultiExpr for YieldNode {
    fn values(&self) -> &Vec<Expr> {
        &self.values
    }

    fn values_mut(&mut self) -> &mut Vec<Expr> {
        &mut self.values
    }

    fn name() -> &'static str {
        "yield"
    }
}
impl std::fmt::Display for YieldNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", MultiExpr::to_string(self))
    }
}
