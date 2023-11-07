mod insert_call;
mod insert_func;
mod insert_phi;
mod make_ssa;

use tohdl_ir::graph::DiGraph;

pub trait Transform {
    fn transform(&self, graph: &mut DiGraph);
}

#[cfg(test)]
pub(crate) mod tests {
    use tohdl_ir::expr::*;
    use tohdl_ir::graph::*;

    pub fn write_graph(graph: &DiGraph, path: &str) {
        // Write dot to file
        use std::fs::File;
        use std::io::Write;
        let mut file = File::create(path).unwrap();
        file.write_all(graph.to_dot().as_bytes()).unwrap();
    }

    /// Make range function
    pub fn make_range() -> graph::DiGraph {
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

        graph
    }
}
