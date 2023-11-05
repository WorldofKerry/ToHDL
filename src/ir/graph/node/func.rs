use crate::ir::expr::VarExpr;

#[derive(Clone)]
pub struct FuncNode {
    pub args: Vec<VarExpr>,
}

impl std::fmt::Display for FuncNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let args = self
            .args
            .iter()
            .map(|a| format!("{}", a))
            .collect::<Vec<_>>()
            .join(", ");
        return write!(f, "func({})", args);
    }
}
