#[derive(Debug, Clone)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone)]
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

#[derive(Clone)]
pub struct IntExpr {
    pub value: i32,
}

impl std::fmt::Debug for IntExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

#[derive(Clone)]
pub enum Expr {
    Var(VarExpr),
    Int(IntExpr),
    Add(BinOpExpr),
}

impl std::fmt::Debug for VarExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Var(var) => write!(f, "{:?}", var.name),
            Expr::Int(int) => write!(f, "{:?}", int.value),
            Expr::Add(add) => write!(f, "{:?} + {:?}", add.lhs, add.rhs),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BinOpExpr {
    pub lhs: Box<Expr>,
    pub oper: Operator,
    pub rhs: Box<Expr>,
}
