use super::*;
use ast::*;
use rustpython_parser::ast::Visitor;
use rustpython_parser::{ast, Parse};
use tohdl_ir::expr::Expr;
use tohdl_ir::graph::{DiGraph, NodeIndex};

struct MyVisitor {
    graph: DiGraph,
    expr_stack: Vec<tohdl_ir::expr::Expr>,
    node_stack: Vec<NodeIndex>,
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

impl MyVisitor {
    fn get_graph(&self) -> DiGraph {
        self.graph.clone()
    }

    /// Returns the last node in the node stack
    /// Restores stack to before the nest was created
    fn visit_nested() -> NodeIndex {
        todo!()
    }

    pub fn debug_status(&self) -> String {
        format!(
            "expr_stack {:?}, node_stack {:?}",
            self.expr_stack, self.node_stack
        )
    }

    pub fn print_debug_status(&self) {
        println!("{}", self.debug_status());
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
        let prev = self.node_stack.pop().unwrap();
        self.graph.add_edge(prev, node, tohdl_ir::graph::Edge::None);
        self.node_stack.push(node);
        self.print_debug_status();
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
    fn visit_expr_compare(&mut self, node: ExprCompare) {
        let op = match node.ops[0] {
            CmpOp::Lt => tohdl_ir::expr::Operator::Lt,
            CmpOp::Gt => tohdl_ir::expr::Operator::Gt,
            _ => todo!(),
        };
        self.generic_visit_expr_compare(node);
        let right = self.expr_stack.pop().unwrap();
        let left = self.expr_stack.pop().unwrap();
        let expr = tohdl_ir::expr::Expr::BinOp(Box::new(left), op, Box::new(right));
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
    fn visit_stmt_if(&mut self, node: StmtIf) {
        let prev = self.node_stack.last().unwrap().clone();
        {
            let value = node.test;
            self.visit_expr(*value);
        }
        let condition = self.expr_stack.pop().unwrap();
        println!("condition {:?}", condition);
        self.print_debug_status();
        let ifelse = tohdl_ir::graph::Node::Branch(tohdl_ir::graph::BranchNode { cond: condition });
        let ifelse_node = self.graph.add_node(ifelse);
        self.node_stack.push(ifelse_node);

        for value in node.body {
            self.visit_stmt(value);
        }
        let true_final = self.node_stack.pop().unwrap();
        for value in node.orelse {
            todo!();
            self.visit_stmt(value);
        }
        self.graph
            .add_edge(prev, ifelse_node, tohdl_ir::graph::Edge::None);
        self.node_stack.push(true_final);
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
        i = i + 1
"#;
        let python_source = r#"
def func(n):
    i = n + 10
    j = 10 + 15
    if j > 10:
        n = 100
        n = 150
"#;
        let mut visitor = MyVisitor::default();
        let ast = ast::Suite::parse(python_source, "<embedded>").unwrap();

        println!("ast {:#?}", ast);
        visitor.visit_stmt(ast[0].clone());

        let graph = visitor.get_graph();

        println!("graph {}", graph.to_dot());
        graph.write_dot("visitor.dot")
    }
}
