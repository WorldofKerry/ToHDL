use crate::expr::VarExpr;

#[derive(Clone, PartialEq)]
pub struct CallNode {
    pub params: Vec<VarExpr>,
}

impl std::fmt::Display for CallNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let params = self
            .params
            .iter()
            .map(|a| format!("{}", a))
            .collect::<Vec<_>>()
            .join(", ");
        return write!(f, "call({})", params);
    }
}
