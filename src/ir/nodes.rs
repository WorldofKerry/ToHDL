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

struct Graph(pub petgraph::Graph<Node, Option<bool>, petgraph::Directed, u32>);

impl std::ops::Deref for Graph {
    type Target = petgraph::Graph<Node, Option<bool>, petgraph::Directed, u32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Graph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Graph {
    fn to_dot(&self) -> String {
        format!(
            "{:?}",
            petgraph::dot::Dot::with_config(&self.0, &[petgraph::dot::Config::EdgeNoLabel])
        )
    }
}

#[derive(Debug, Clone)]
pub struct Blank;

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn graph() {
        let mut graph = Graph(petgraph::Graph::new());

        let x = VarExpr::new("x");

        let n0 = graph.add_node(Node::Assign(AssignNode {
            lvalue: x.clone(),
            rvalue: Expr::Int(IntExpr { value: 1 }),
        }));
        let n1 = graph.add_node(Node::Assign(AssignNode {
            lvalue: x.clone(),
            rvalue: Expr::Int(IntExpr { value: 1 }),
        }));
        graph.add_edge(n0, n1, None);

        print!("{}", graph.to_dot());

        // Write dot to file
        use std::fs::File;
        use std::io::Write;
        let mut file = File::create("graph.dot").unwrap();
        file.write_all(graph.to_dot().as_bytes()).unwrap();
    }
}
