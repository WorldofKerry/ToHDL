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

impl std::fmt::Debug for VarExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.name)
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
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub struct BinOpExpr {
    pub lhs: Box<Expr>,
    pub oper: Operator,
    pub rhs: Box<Expr>,
}

#[derive(Clone)]
pub struct AssignNode {
    pub lvalue: VarExpr,
    pub rvalue: Expr,
}

impl std::fmt::Debug for AssignNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?} = {:?}", self.lvalue, self.rvalue)
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    Assign(AssignNode),
}
