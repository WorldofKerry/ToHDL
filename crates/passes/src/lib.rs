pub mod algorithms;
pub mod manager;
pub mod optimize;
pub mod transform;

use tohdl_ir::graph::CFG;

#[derive(Debug, Clone)]
pub struct TransformResultType {
    pub elapsed_time: std::time::Duration,
    pub name: String,
    did_work: bool,
}

impl Default for TransformResultType {
    fn default() -> Self {
        Self { elapsed_time: Default::default(), name: Default::default(), did_work: true }
    }
}

impl TransformResultType {
    pub fn no_work() -> Self {
        let mut result = Self::default();
        result.did_work = false;
        result
    }

    /// Signal that work was done
    pub fn did_work(&mut self) {
        self.did_work = true;
    }
}


impl std::fmt::Display for TransformResultType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Transform {:>20} elapsed time: {:>12?}", self.name, self.elapsed_time)
    }
}

pub trait BasicTransform: Default {
    /// Applies transform on a graph.
    /// Prefer [Transform::transform] to avoid having to create a temporary.
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType;

    /// Applies transform on a graph.
    /// Prefer [Transform::transform] to avoid having to create a temporary.
    fn apply_timed(&mut self, graph: &mut CFG) -> TransformResultType
    where
        Self: Sized,
    {
        <Self as ContextfulTransfrom<()>>::apply_timed(self, graph, &mut ())
    }

    /// Applies transform on a graph
    fn transform(graph: &mut CFG) -> TransformResultType
    where
        Self: Sized,
    {
        <Self as ContextfulTransfrom<()>>::transform_contextful(graph, &mut ())
    }

    /// Name of transform
    fn name(&self) -> &str {
        <Self as ContextfulTransfrom<()>>::name_contextful(self)
    }
}

pub trait ContextfulTransfrom<Context>: Default {
    /// Name of transform
    fn name_contextful(&self) -> &str {
        std::any::type_name::<Self>().rsplit_once("::").unwrap().1
    }
    fn apply_contextful(&mut self, graph: &mut CFG, context: &mut Context) -> &TransformResultType;

    /// Applies transform on a graph.
    /// Prefer [Transform::transform] to avoid having to create a temporary.
    fn apply_timed(&mut self, graph: &mut CFG, context: &mut Context) -> TransformResultType
    where
        Self: Sized,
    {
        let start_time = std::time::Instant::now();
        let mut result = (*self.apply_contextful(graph, context)).clone();
        result.elapsed_time = start_time.elapsed();
        result.name = self.name_contextful().into();
        result
    }
    fn transform_contextful(graph: &mut CFG, context: &mut Context) -> TransformResultType
    where
        Self: Sized,
    {
        let mut transform = Self::default();
        transform.apply_timed(graph, context)
    }
}

impl<T, Context> ContextfulTransfrom<Context> for T
where
    T: BasicTransform,
{
    fn apply_contextful(&mut self, graph: &mut CFG, _: &mut Context) -> &TransformResultType {
        self.apply(graph)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use tohdl_ir::expr::*;
    use tohdl_ir::graph::*;

    /// Write graph to [path].dot
    pub fn write_graph(graph: &CFG, path: &str) {
        use std::fs::File;
        use std::io::Write;
        use std::path::PathBuf;

        let mut path = PathBuf::from(path);
        path.set_extension("dot");
        let mut file = File::create(path).unwrap();
        file.write_all(graph.to_dot().as_bytes()).unwrap();
    }

    /// Make range function
    pub fn make_range() -> CFG {
        let mut graph = CFG::default();

        let i = VarExpr::new("i");
        let n = VarExpr::new("n");

        // func(n)
        let entry = graph.add_node(FuncNode {
            params: vec![n.clone()],
        });

        let n0 = graph.add_node(AssignNode {
            lvalue: i.clone(),
            rvalue: Expr::Int(IntExpr::new(0)),
        });
        graph.add_edge(entry, n0, NoneEdge.into());

        let n1 = graph.add_node(BranchNode {
            cond: Expr::BinOp(
                Box::new(Expr::Var(i.clone())),
                Operator::Lt,
                Box::new(Expr::Var(n.clone())),
            ),
        });

        graph.add_edge(n0, n1, NoneEdge.into());

        // Loop body
        let t0 = graph.add_node(AssignNode {
            lvalue: i.clone(),
            rvalue: Expr::BinOp(
                Box::new(Expr::Var(i.clone())),
                Operator::Add,
                Box::new(Expr::Int(IntExpr::new(1))),
            ),
        });
        graph.add_edge(n1, t0, BranchEdge::new(true).into());

        let t1 = graph.add_node(ReturnNode {
            values: vec![Expr::Var(i.clone())],
        });
        graph.add_edge(t0, t1, NoneEdge.into());

        graph.add_edge(t1, n1, NoneEdge.into());

        // Loop end
        let f0 = graph.add_node(ReturnNode { values: vec![] });
        graph.add_edge(n1, f0, BranchEdge::new(false).into());

        graph
    }

    /// Make fib function
    pub fn make_fib() -> CFG {
        let mut graph = CFG::default();

        let n = VarExpr::new("n");
        let a = VarExpr::new("a");
        let b = VarExpr::new("b");
        let i = VarExpr::new("i");
        let temp = VarExpr::new("temp");

        // func(n)
        let entry = graph.add_node(FuncNode {
            params: vec![n.clone()],
        });

        // a = 0
        let n0 = graph.add_node(AssignNode {
            lvalue: a.clone(),
            rvalue: Expr::Int(IntExpr::new(0)),
        });
        graph.add_edge(entry, n0, NoneEdge.into());

        // b = 1
        let n1 = graph.add_node(AssignNode {
            lvalue: b.clone(),
            rvalue: Expr::Int(IntExpr::new(1)),
        });
        graph.add_edge(n0, n1, NoneEdge.into());

        // i = 0
        let n2 = graph.add_node(AssignNode {
            lvalue: i.clone(),
            rvalue: Expr::Int(IntExpr::new(0)),
        });
        graph.add_edge(n1, n2, NoneEdge.into());

        // if i < n
        let n3 = graph.add_node(BranchNode {
            cond: Expr::BinOp(
                Box::new(Expr::Var(i.clone())),
                Operator::Lt,
                Box::new(Expr::Var(n.clone())),
            ),
        });
        graph.add_edge(n2, n3, NoneEdge.into());

        // true branch
        // temp = a + b
        let t0 = graph.add_node(AssignNode {
            lvalue: temp.clone(),
            rvalue: Expr::BinOp(
                Box::new(Expr::Var(a.clone())),
                Operator::Add,
                Box::new(Expr::Var(b.clone())),
            ),
        });
        graph.add_edge(n3, t0, BranchEdge::new(true).into());

        // a = b
        let t1 = graph.add_node(AssignNode {
            lvalue: a.clone(),
            rvalue: Expr::Var(b.clone()),
        });
        graph.add_edge(t0, t1, NoneEdge.into());

        // b = temp
        let t2 = graph.add_node(AssignNode {
            lvalue: b.clone(),
            rvalue: Expr::Var(temp.clone()),
        });
        graph.add_edge(t1, t2, NoneEdge.into());

        // i = i + 1
        let t3 = graph.add_node(AssignNode {
            lvalue: i.clone(),
            rvalue: Expr::BinOp(
                Box::new(Expr::Var(i.clone())),
                Operator::Add,
                Box::new(Expr::Int(IntExpr::new(1))),
            ),
        });
        graph.add_edge(t2, t3, NoneEdge.into());

        // yield a
        let t4 = graph.add_node(ReturnNode {
            values: vec![Expr::Var(a.clone())],
        });
        graph.add_edge(t3, t4, NoneEdge.into());

        // loop
        graph.add_edge(t4, n3, NoneEdge.into());

        // false branch
        // return
        let f0 = graph.add_node(ReturnNode { values: vec![] });
        graph.add_edge(n3, f0, BranchEdge::new(false).into());

        graph
    }

    /// Make branch
    pub fn make_branch() -> CFG {
        let mut graph = CFG::default();

        let a = VarExpr::new("a");
        let b = VarExpr::new("b");

        // func(a)
        let entry = graph.add_node(FuncNode {
            params: vec![a.clone()],
        });

        // if a < 10
        let n0 = graph.add_node(BranchNode {
            cond: Expr::BinOp(
                Box::new(Expr::Var(a.clone())),
                Operator::Lt,
                Box::new(Expr::Int(IntExpr::new(10))),
            ),
        });
        graph.add_edge(entry, n0, NoneEdge.into());

        // true branch
        // b = 1
        let t0 = graph.add_node(AssignNode {
            lvalue: b.clone(),
            rvalue: Expr::Int(IntExpr::new(1)),
        });
        graph.add_edge(n0, t0, BranchEdge::new(true).into());

        // false branch
        // b = 2
        let f0 = graph.add_node(AssignNode {
            lvalue: b.clone(),
            rvalue: Expr::Var(a.clone()),
        });
        graph.add_edge(n0, f0, BranchEdge::new(false).into());

        // return 0
        let n1 = graph.add_node(ReturnNode {
            values: vec![Expr::Var(b.clone())],
        });
        graph.add_edge(t0, n1, NoneEdge.into());
        graph.add_edge(f0, n1, NoneEdge.into());

        graph
    }

    /// Make odd fib
    pub fn make_even_fib() -> CFG {
        let code = r#"
def even_fib(n):
    i = 0
    a = 0
    b = 1
    while i < n:
        if a % 2:
            yield a
        temp = a + b
        a = b
        b = temp
        i = i + 1
    return 0
"#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);

        let graph = visitor.get_graph();

        graph
    }

    pub fn make_double_while() -> CFG {
        let code = r#"
def double_while(n):
    x = 0
    while x < n:
        y = 0
        while y < n:
            yield x
            y = y + 1
        x = x + 1
    return 0
"#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);

        let graph = visitor.get_graph();

        graph
    }

    pub fn make_linear() -> CFG {
        let code = r#"
def linear():
    a = 42
    b = a
    c = a + b
    a = c + 23
    c = a + d
    return 0
        "#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);

        let graph = visitor.get_graph();

        graph
    }

    pub fn make_complex_branch() -> CFG {
        let code = r#"
def func(a):
    if a < 10:
        b = 1
    else:
"#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);

        let graph = visitor.get_graph();

        graph
    }

    pub fn make_aug_assign() -> CFG {
        let code = r#"
def aug_assign(a):
    a += 5
    return a
"#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);

        let graph = visitor.get_graph();

        graph
    }
}
