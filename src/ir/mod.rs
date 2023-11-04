pub mod expr;
pub mod graph;

#[cfg(test)]

mod tests {
    use super::expr::*;
    use super::graph::*;

    #[test]
    fn graph() {
        let mut graph = Graph(petgraph::Graph::new());

        let i = VarExpr::new("i");
        let n = VarExpr::new("n");

        let n0 = graph.add_node(Node::Assign(AssignNode {
            lvalue: i.clone(),
            rvalue: Expr::Int(IntExpr::new(0)),
        }));

        let n1 = graph.add_node(Node::Branch(BranchNode {
            cond: Expr::BinOp(BinOpExpr::new(
                Expr::Var(i.clone()),
                Operator::Lt,
                Expr::Var(n.clone()),
            )),
        }));

        graph.add_edge(n0, n1, Edge::None);

        let n2 = graph.add_node(Node::Assign(AssignNode {
            lvalue: i.clone(),
            rvalue: Expr::BinOp(BinOpExpr::new(
                Expr::Var(i),
                Operator::Add,
                Expr::Int(IntExpr::new(1)),
            )),
        }));

        graph.add_edge(n1, n2, Edge::Branch(true));
        graph.add_edge(n2, n1, Edge::None);

        print!("{}", graph.to_dot());

        // Write dot to file
        use std::fs::File;
        use std::io::Write;
        let mut file = File::create("graph.dot").unwrap();
        file.write_all(graph.to_dot().as_bytes()).unwrap();

    }
}
