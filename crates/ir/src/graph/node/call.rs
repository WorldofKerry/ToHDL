use crate::expr::VarExpr;

use super::ReadsVariables;

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
        return write!(f, "call({})", params);
    }
}

impl ReadsVariables for CallNode {
    fn read_vars(&mut self) -> Vec<&mut VarExpr> {
        self.args.iter_mut().collect()
    }
}
