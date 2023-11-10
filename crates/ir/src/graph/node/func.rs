use crate::expr::VarExpr;

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
        return write!(f, "func({})", args);
    }
}
