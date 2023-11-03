// use rustpython_parser::ast::{Constant, Expr, ExprConstant, ExprContext, ExprName};

#[derive(Debug, Clone)]
pub struct VarExpr {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct IntExpr {
    pub value: i32,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Var(VarExpr),
    Int(IntExpr),
    Add(AddExpr),
}

#[derive(Debug, Clone)]
pub struct AddExpr {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct AssignNode {
    pub lvalue: VarExpr,
    pub rvalue: Expr,
}

#[derive(Debug, Clone)]
pub enum Node {
    Assign(AssignNode),
}

struct Graph(pub petgraph::Graph<Node, (), petgraph::Directed, u32>);

impl std::ops::Deref for Graph {
    type Target = petgraph::Graph<Node, (), petgraph::Directed, u32>;

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
    use rustpython_parser::{ast, Parse};

    #[test]
    fn graph() {
        let mut graph = Graph(petgraph::Graph::new());

        graph.add_node(Node::Assign(AssignNode {
            lvalue: VarExpr {
                name: "x".to_string(),
            },
            rvalue: Expr::Int(IntExpr { value: 10 }),
        }));

        print!("{}", graph.to_dot());
    }
}
