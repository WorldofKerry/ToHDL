use super::*;
use ast::*;
use rustpython_parser::ast::Visitor;
use rustpython_parser::{ast, Parse};
use tohdl_ir::expr::Expr;
use tohdl_ir::graph::DiGraph;

struct MyVisitor {
    graph: DiGraph,
    expr_stack: Vec<tohdl_ir::expr::Expr>,
    node_stack: Vec<usize>,
}

impl MyVisitor {
    fn get_graph(&self) -> DiGraph {
        self.graph.clone()
    }
}

impl Default for MyVisitor {
    fn default() -> Self {
        let mut ret = Self {
            graph: DiGraph::new(),
            expr_stack: vec![],
            node_stack: vec![],
        };

        // Initialize root func node
        let node = tohdl_ir::graph::Node::Func(tohdl_ir::graph::FuncNode { params: vec![] });
        let root = ret.graph.add_node(node);
        ret.node_stack.push(root);

        ret
    }
}

impl Visitor for MyVisitor {
    fn visit_stmt_assign(&mut self, node: StmtAssign) {
        for value in node.targets {
            self.visit_expr(value);
        }
        let target = match self.expr_stack.pop().unwrap() {
            tohdl_ir::expr::Expr::Var(var) => var,
            _ => todo!(),
        };
        {
            let value = node.value;
            self.visit_expr(*value);
        }
        let value = self.expr_stack.pop().unwrap();
        let node = tohdl_ir::graph::Node::Assign(tohdl_ir::graph::AssignNode {
            lvalue: target,
            rvalue: value,
        });
        let node = self.graph.add_node(node);
        let prev = self.node_stack.last().unwrap();
        self.graph
            .add_edge(*prev, node, tohdl_ir::graph::Edge::None);
        self.node_stack.push(node);
    }
    fn visit_expr_bin_op(&mut self, node: ExprBinOp) {
        // println!("visit_expr_bin_op {:?}", node);
        self.generic_visit_expr_bin_op(node);
        let right = self.expr_stack.pop().unwrap();
        let left = self.expr_stack.pop().unwrap();
        let expr = tohdl_ir::expr::Expr::BinOp(
            Box::new(left),
            tohdl_ir::expr::Operator::Add,
            Box::new(right),
        );
        self.expr_stack.push(expr);
    }
    fn visit_expr_name(&mut self, node: ExprName) {
        self.expr_stack
            .push(tohdl_ir::expr::Expr::Var(tohdl_ir::expr::VarExpr::new(
                node.id.as_str(),
            )));
    }
    fn visit_expr_constant(&mut self, node: ExprConstant) {
        self.expr_stack.push(match node.value {
            Constant::Int(i) => tohdl_ir::expr::Expr::Int(tohdl_ir::expr::IntExpr::new(
                str::parse::<i32>(&i.to_string()).unwrap(),
            )),
            _ => todo!(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visitor() {
        let python_source = r#"
def rectangle(m, n):
    i = 0
    while i < m:
        j = 0
        while j < n:
            if i % 2:
                test = 1 + (2 + 3) + (4 + 5) * 6
                yield i, j
            j = j + 1
        print()
        i = i + 1
"#;
        let python_source = r#"
i = n + 10
"#;
        let mut visitor = MyVisitor::default();
        let ast = ast::Suite::parse(python_source, "<embedded>").unwrap();

        // println!("{:#?}", ast);
        visitor.visit_stmt(ast[0].clone());

        let graph = visitor.get_graph();

        println!("graph {}", graph.to_dot());
    }
}
