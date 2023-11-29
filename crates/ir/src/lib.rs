pub mod expr;
pub mod graph;

#[cfg(test)]

pub(crate) mod tests {
    use super::expr::*;
    use super::graph::*;

    pub fn write_graph(graph: &CFG, path: &str) {
        // Write dot to file
        use std::fs::File;
        use std::io::Write;
        let mut file = File::create(path).unwrap();
        file.write_all(graph.to_dot().as_bytes()).unwrap();
    }

    /// Make range function
    pub fn make_range() -> CFG {
        let mut graph = CFG::default();

        let i = VarExpr::new("i");
        let n = VarExpr::new("n");

        let n0 = graph.add_node(AssignNode {
            lvalue: i.clone(),
            rvalue: Expr::Int(IntExpr::new(0)),
        });

        let n1 = graph.add_node(BranchNode {
            cond: Expr::BinOp(
                Box::new(Expr::Var(i.clone())),
                Operator::Lt,
                Box::new(Expr::Var(n.clone())),
            ),
        });

        graph.add_edge(n0, n1, Edge::None);

        // Loop body
        let t0 = graph.add_node(AssignNode {
            lvalue: i.clone(),
            rvalue: Expr::BinOp(
                Box::new(Expr::Var(i.clone())),
                Operator::Add,
                Box::new(Expr::Int(IntExpr::new(1))),
            ),
        });
        graph.add_edge(n1, t0, Edge::Branch(true));

        let t1 = graph.add_node(TermNode {
            values: vec![Expr::Var(i)],
        });
        graph.add_edge(t0, t1, Edge::None);

        graph.add_edge(t1, n1, Edge::None);

        // Loop end
        let f0 = graph.add_node(TermNode { values: vec![] });
        graph.add_edge(n1, f0, Edge::Branch(false));

        graph
    }

    #[test]
    fn graph() {
        let graph = make_range();

        print!("{}", graph.to_dot());

        graph.write_dot("graph.dot");
    }
}
