pub mod expr;
pub mod graph;
pub mod pass;

#[cfg(test)]

mod tests {
    use crate::ir::pass::Transform;

    use super::expr::*;
    use super::graph::*;

    #[test]
    fn graph() {
        let mut graph = DiGraph(petgraph::Graph::new());

        let i = VarExpr::new("i");
        let n = VarExpr::new("n");

        let n0 = graph.add_node(Node::Assign(AssignNode {
            lvalue: i.clone(),
            rvalue: Expr::Int(IntExpr::new(0)),
        }));

        let n1 = graph.add_node(Node::Branch(BranchNode {
            cond: Expr::BinOp(
                Box::new(Expr::Var(i.clone())),
                Operator::Lt,
                Box::new(Expr::Var(n.clone())),
            ),
        }));

        graph.add_edge(n0, n1, Edge::None);

        // Loop body
        let t0 = graph.add_node(Node::Assign(AssignNode {
            lvalue: i.clone(),
            rvalue: Expr::BinOp(
                Box::new(Expr::Var(i.clone())),
                Operator::Add,
                Box::new(Expr::Int(IntExpr::new(1))),
            ),
        }));
        graph.add_edge(n1, t0, Edge::Branch(true));

        let t1 = graph.add_node(Node::Yield(TermNode { values: vec![] }));
        graph.add_edge(t0, t1, Edge::None);

        graph.add_edge(t1, n1, Edge::None);

        // Loop end
        let f0 = graph.add_node(Node::Return(TermNode { values: vec![] }));
        graph.add_edge(n1, f0, Edge::Branch(false));

        // Apply transformations
        let trans = crate::ir::pass::insert_func::InsertFuncNodes {};
        trans.transform(&mut graph);

        // Write dot to file
        use std::fs::File;
        use std::io::Write;
        let mut file = File::create("graph.dot").unwrap();
        file.write_all(graph.to_dot().as_bytes()).unwrap();
    }
}
