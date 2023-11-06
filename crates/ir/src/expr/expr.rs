#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Gt,
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Operator::Add => write!(f, "+"),
            Operator::Sub => write!(f, "-"),
            Operator::Mul => write!(f, "*"),
            Operator::Div => write!(f, "/"),
            Operator::Lt => write!(f, "<"),
            Operator::Gt => write!(f, ">"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VarExpr {
    pub name: String,
}

impl VarExpr {
    pub fn new(name: &str) -> Self {
        VarExpr {
            name: name.to_string(),
        }
    }
}

impl std::fmt::Display for VarExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IntExpr {
    pub value: i32,
}

impl IntExpr {
    pub fn new(value: i32) -> Self {
        IntExpr { value }
    }
}

impl std::fmt::Display for IntExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Var(VarExpr),
    Int(IntExpr),
    BinOp(Box<Expr>, Operator, Box<Expr>),
}

impl Expr {
    /// Recursively get all variables referenced in the expression
    pub fn get_vars(&self) -> Vec<VarExpr> {
        match self {
            Expr::Var(var) => vec![var.clone()],
            Expr::Int(_) => vec![],
            Expr::BinOp(left, _, right) => {
                let mut ret = left.get_vars();
                ret.extend(right.get_vars());
                ret
            }
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Var(e) => write!(f, "{}", e),
            Expr::Int(e) => write!(f, "{}", e),
            Expr::BinOp(left, op, right) => write!(f, "({} {} {})", left, op, right),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr() {
        let expr = Expr::BinOp(
            Box::new(Expr::Var(VarExpr::new("a"))),
            Operator::Add,
            Box::new(Expr::Var(VarExpr::new("b"))),
        );

        assert_eq!(expr.to_string(), "(a + b)");

        assert_eq!(expr.get_vars(), vec![VarExpr::new("a"), VarExpr::new("b")]);
    }
}
