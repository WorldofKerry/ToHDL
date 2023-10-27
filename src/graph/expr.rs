pub struct Var {}

impl std::fmt::Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Var")
    }
}

pub struct Expression {}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Expression")
    }
}
