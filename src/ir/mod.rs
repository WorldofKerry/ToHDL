pub mod expr;
pub mod graph;

#[cfg(test)]

mod tests {
    use super::expr::*;
    use super::graph::*;
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
