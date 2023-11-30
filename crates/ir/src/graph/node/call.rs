use crate::expr::VarExpr;

use super::DataFlow;

#[derive(Clone, PartialEq)]
pub struct CallNode {
    pub args: Vec<VarExpr>,
}

impl std::fmt::Display for CallNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let params = self
            .args
            .iter()
            .map(|a| format!("{}", a))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "call({})", params)
    }
}

impl DataFlow for CallNode {
    fn referenced_vars(&self) -> Vec<&VarExpr> {
        self.args.iter().collect()
    }
    fn referenced_vars_mut(&mut self) -> Vec<&mut VarExpr> {
        self.args.iter_mut().collect()
    }
    fn defined_vars(&self) -> std::collections::BTreeMap<&VarExpr, &crate::expr::Expr> {
        panic!("This function should never be called for Call/Func nodes. They should have been previously replaced with phi nodes.")
    }
}
