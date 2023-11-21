use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
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
            Operator::Mod => write!(f, "%"),
            Operator::Lt => write!(f, "<"),
            Operator::Gt => write!(f, ">"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    pub fn get_vars(&self) -> Vec<&VarExpr> {
        match self {
            Expr::Var(var) => vec![var],
            Expr::Int(_) => vec![],
            Expr::BinOp(left, _, right) => {
                let mut ret = left.get_vars();
                ret.extend(right.get_vars());
                ret
            }
        }
    }

    /// Recursively iterate over all variables referenced in the expression
    pub fn get_vars_iter(&mut self) -> impl Iterator<Item = &mut VarExpr> {
        let result: Box<dyn Iterator<Item = &mut VarExpr>> = match self {
            Expr::Var(var) => Box::new(std::iter::once(var)),
            Expr::Int(_) => Box::new(std::iter::empty::<&mut VarExpr>()),
            Expr::BinOp(left, _, right) => {
                Box::new(left.get_vars_iter().chain(right.get_vars_iter()))
            }
        };
        result
    }

    /// Recursively iterate over all variables referenced in the expression as a Expr
    pub fn get_exprs_iter(&mut self) -> impl Iterator<Item = &mut Expr> {
        let result: Box<dyn Iterator<Item = &mut Expr>> = match self {
            Expr::Var(_) => Box::new(std::iter::once(self)),
            Expr::Int(_) => Box::new(std::iter::empty::<&mut Expr>()),
            Expr::BinOp(left, _, right) => {
                Box::new(left.get_exprs_iter().chain(right.get_exprs_iter()))
            }
        };
        result
    }

    /// Recursively replace variables with mapped expression
    pub fn backwards_replace(&mut self, mapping: &BTreeMap<VarExpr, Expr>) {
        for expr in self.get_exprs_iter() {
            if let Expr::Var(var) = expr {
                if let Some(replacement) = mapping.get(var) {
                    *expr = replacement.clone();
                } else {
                    // panic!(
                    //     "backwards_replace: Variable {} not found in mapping {:?}",
                    //     var, mapping
                    // );
                    println!(
                        "backwards_replace: Variable {} not found in mapping {:?}",
                        var, mapping
                    );
                }
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
    fn test_iter_vars_mutate() {
        let mut expr = Expr::BinOp(
            Box::new(Expr::Var(VarExpr::new("a"))),
            Operator::Add,
            Box::new(Expr::Var(VarExpr::new("b"))),
        );

        for var in expr.get_vars_iter() {
            var.name = "c".to_string();
        }

        assert_eq!(expr.to_string(), "(c + c)");
    }

    #[test]
    fn test_expr() {
        let expr = Expr::BinOp(
            Box::new(Expr::Var(VarExpr::new("a"))),
            Operator::Add,
            Box::new(Expr::Var(VarExpr::new("b"))),
        );

        assert_eq!(expr.to_string(), "(a + b)");

        assert_eq!(
            expr.get_vars(),
            vec![&VarExpr::new("a"), &VarExpr::new("b")]
        );
    }

    #[test]
    fn test_backwards_replace() {
        // a + ((b + a) + c)
        let mut expr = Expr::BinOp(
            Box::new(Expr::Var(VarExpr::new("a"))),
            Operator::Add,
            Box::new(Expr::BinOp(
                Box::new(Expr::BinOp(
                    Box::new(Expr::Var(VarExpr::new("b"))),
                    Operator::Add,
                    Box::new(Expr::Var(VarExpr::new("a"))),
                )),
                Operator::Add,
                Box::new(Expr::Var(VarExpr::new("c"))),
            )),
        );

        // a -> 10
        let mapping: BTreeMap<VarExpr, Expr> = vec![
            (VarExpr::new("a"), Expr::Int(IntExpr::new(10))),
            (VarExpr::new("b"), Expr::Var(VarExpr::new("b"))),
            (VarExpr::new("c"), Expr::Var(VarExpr::new("c"))),
        ]
        .into_iter()
        .collect();

        expr.backwards_replace(&mapping);

        println!("result {}", expr);

        assert_eq!(expr.to_string(), "(10 + ((b + 10) + c))");
    }
}
