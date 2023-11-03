pub mod expr;
pub mod graph;

#[cfg(test)]

mod tests {
    use super::expr::*;
    use super::graph::*;

    #[test]
    fn graph() {
        let mut graph = Graph(petgraph::Graph::new());

        let x = VarExpr::new("x");
        let n = VarExpr::new("n");

        let n0 = graph.add_node(Node::Assign(AssignNode {
            lvalue: x.clone(),
            rvalue: Expr::Int(IntExpr::new(0)),
        }));

        let n1 = graph.add_node(Node::Branch(BranchNode {
            cond: Expr::BinOp(BinOpExpr {
                op: Operator::Lt,
                lhs: Box::new(Expr::Var(x.clone())),
                rhs: Box::new(Expr::BinOp(BinOpExpr {
                    lhs: Box::new(Expr::Var(n.clone())),
                    op: Operator::Add,
                    rhs: Box::new(Expr::Int(IntExpr::new(1))),
                })),
            }),
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
