pub mod insert_call;
pub mod insert_func;
pub mod insert_phi;
pub mod lower_to_fsm;
pub mod make_ssa;
pub mod manager;

use tohdl_ir::graph::DiGraph;

pub struct TransformResultType {
    did_work: bool,
}

impl Default for TransformResultType {
    fn default() -> Self {
        Self { did_work: false }
    }
}

pub trait Transform: Default {
    // fn transform(&mut self, graph: &mut DiGraph) -> TransformResultType;
    fn apply(&mut self, graph: &mut DiGraph);
    fn transform(graph: &mut DiGraph)
    where
        Self: Sized,
    {
        let mut transform = Self::default();
        transform.apply(graph);
    }
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

        // func(n)
        let entry = graph.add_node(Node::Func(FuncNode {
            params: vec![n.clone()],
        }));

        let n0 = graph.add_node(Node::Assign(AssignNode {
            lvalue: i.clone(),
            rvalue: Expr::Int(IntExpr::new(0)),
        }));
        graph.add_edge(entry, n0, Edge::None);

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

    /// Make fib function
    pub fn make_fib() -> graph::DiGraph {
        let mut graph = DiGraph(petgraph::Graph::new());

        let n = VarExpr::new("n");
        let a = VarExpr::new("a");
        let b = VarExpr::new("b");
        let i = VarExpr::new("i");
        let temp = VarExpr::new("temp");

        // func(n)
        let entry = graph.add_node(Node::Func(FuncNode {
            params: vec![n.clone()],
        }));

        // a = 0
        let n0 = graph.add_node(Node::Assign(AssignNode {
            lvalue: a.clone(),
            rvalue: Expr::Int(IntExpr::new(0)),
        }));
        graph.add_edge(entry, n0, Edge::None);

        // b = 1
        let n1 = graph.add_node(Node::Assign(AssignNode {
            lvalue: b.clone(),
            rvalue: Expr::Int(IntExpr::new(1)),
        }));
        graph.add_edge(n0, n1, Edge::None);

        // i = 0
        let n2 = graph.add_node(Node::Assign(AssignNode {
            lvalue: i.clone(),
            rvalue: Expr::Int(IntExpr::new(0)),
        }));
        graph.add_edge(n1, n2, Edge::None);

        // if i < n
        let n3 = graph.add_node(Node::Branch(BranchNode {
            cond: Expr::BinOp(
                Box::new(Expr::Var(i.clone())),
                Operator::Lt,
                Box::new(Expr::Var(n.clone())),
            ),
        }));
        graph.add_edge(n2, n3, Edge::None);

        // true branch
        // temp = a + b
        let t0 = graph.add_node(Node::Assign(AssignNode {
            lvalue: temp.clone(),
            rvalue: Expr::BinOp(
                Box::new(Expr::Var(a.clone())),
                Operator::Add,
                Box::new(Expr::Var(b.clone())),
            ),
        }));
        graph.add_edge(n3, t0, Edge::Branch(true));

        // a = b
        let t1 = graph.add_node(Node::Assign(AssignNode {
            lvalue: a.clone(),
            rvalue: Expr::Var(b.clone()),
        }));
        graph.add_edge(t0, t1, Edge::None);

        // b = temp
        let t2 = graph.add_node(Node::Assign(AssignNode {
            lvalue: b.clone(),
            rvalue: Expr::Var(temp.clone()),
        }));
        graph.add_edge(t1, t2, Edge::None);

        // i = i + 1
        let t3 = graph.add_node(Node::Assign(AssignNode {
            lvalue: i.clone(),
            rvalue: Expr::BinOp(
                Box::new(Expr::Var(i.clone())),
                Operator::Add,
                Box::new(Expr::Int(IntExpr::new(1))),
            ),
        }));
        graph.add_edge(t2, t3, Edge::None);

        // yield a
        let t4 = graph.add_node(Node::Yield(TermNode {
            values: vec![Expr::Var(a.clone())],
        }));
        graph.add_edge(t3, t4, Edge::None);

        // loop
        graph.add_edge(t4, n3, Edge::None);

        // false branch
        // return
        let f0 = graph.add_node(Node::Return(TermNode { values: vec![] }));
        graph.add_edge(n3, f0, Edge::Branch(false));

        graph
    }

    /// Make branch
    pub fn make_branch() -> graph::DiGraph {
        let mut graph = DiGraph::new();

        let a = VarExpr::new("a");
        let b = VarExpr::new("b");

        // func(a)
        let entry = graph.add_node(Node::Func(FuncNode {
            params: vec![a.clone()],
        }));

        // if a < 10
        let n0 = graph.add_node(Node::Branch(BranchNode {
            cond: Expr::BinOp(
                Box::new(Expr::Var(a.clone())),
                Operator::Lt,
                Box::new(Expr::Int(IntExpr::new(10))),
            ),
        }));
        graph.add_edge(entry, n0, Edge::None);

        // true branch
        // b = 1
        let t0 = graph.add_node(Node::Assign(AssignNode {
            lvalue: b.clone(),
            rvalue: Expr::Int(IntExpr::new(1)),
        }));
        graph.add_edge(n0, t0, Edge::Branch(true));

        // false branch
        // b = 2
        let f0 = graph.add_node(Node::Assign(AssignNode {
            lvalue: b.clone(),
            rvalue: Expr::Int(IntExpr::new(2)),
        }));
        graph.add_edge(n0, f0, Edge::Branch(false));

        // return 0
        let n1 = graph.add_node(Node::Return(TermNode {
            values: vec![Expr::Var(b.clone())],
        }));
        graph.add_edge(t0, n1, Edge::None);
        graph.add_edge(f0, n1, Edge::None);

        graph
    }
}
