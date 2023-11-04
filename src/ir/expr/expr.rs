#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub enum Expr {
    Var(VarExpr),
    Int(IntExpr),
    BinOp(BinOpExpr),
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Var(e) => write!(f, "{}", e),
            Expr::Int(e) => write!(f, "{}", e),
            Expr::BinOp(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BinOpExpr {
    pub lhs: Box<Expr>,
    pub op: Operator,
    pub rhs: Box<Expr>,
}

impl std::fmt::Display for BinOpExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} {} {})", self.lhs, self.op, self.rhs)
    }
}

impl BinOpExpr {
    pub fn new(lhs: Expr, op: Operator, rhs: Expr) -> Self {
        BinOpExpr {
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(rhs),
        }
    }
}
