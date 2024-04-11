use ast::*;
use rustpython_parser::ast::Visitor;
use rustpython_parser::{ast, Parse};
use tohdl_ir::expr::VarExpr;
use tohdl_ir::graph::{BranchEdge, Edge, FuncNode, Node, NodeIndex, NoneEdge, CFG};

#[derive(Debug, Clone)]
struct StackEntry {
    node: NodeIndex,
    edge_type: Box<dyn Edge>,
}

impl From<(NodeIndex, Box<dyn Edge>)> for StackEntry {
    fn from((node, edge): (NodeIndex, Box<dyn Edge>)) -> Self {
        Self {
            node,
            edge_type: edge,
        }
    }
}

pub struct AstVisitor {
    graph: CFG,
    expr_stack: Vec<tohdl_ir::expr::Expr>,
    node_stack: Vec<StackEntry>,
}

impl Default for AstVisitor {
    fn default() -> Self {
        let mut ret = Self {
            graph: CFG::default(),
            expr_stack: vec![],
            node_stack: vec![],
        };

        // Initialize root func node
        let node = tohdl_ir::graph::FuncNode { params: vec![] };
        let root = ret.graph.add_node(node);
        ret.node_stack.push((root, NoneEdge.into()).into());

        ret
    }
}

impl AstVisitor {
    pub fn from_text(text: &str) -> Self {
        let mut ret = Self::default();
        let ast = ast::Suite::parse(text, "<embedded>").unwrap();
        // dbg!(&ast);
        ret.visit_stmt(ast[0].clone());
        ret
    }

    pub fn get_graph(&self) -> CFG {
        self.graph.clone()
    }

    pub fn debug_status(&self) -> String {
        format!(
            "expr_stack {:?}, node_stack {:?}",
            self.expr_stack, self.node_stack
        )
    }

    pub fn print_debug_status(&self) {
        // println!("{}", self.debug_status());
    }

    pub fn binop_mapping(op: rustpython_ast::Operator) -> tohdl_ir::expr::Operator {
        match op {
            Operator::Add => tohdl_ir::expr::Operator::Add,
            Operator::Sub => tohdl_ir::expr::Operator::Sub,
            Operator::Mult => tohdl_ir::expr::Operator::Mul,
            Operator::Div => tohdl_ir::expr::Operator::Div,
            Operator::Mod => tohdl_ir::expr::Operator::Mod,
            Operator::LShift => tohdl_ir::expr::Operator::LShift,
            Operator::RShift => tohdl_ir::expr::Operator::RShift,
            Operator::BitAnd => tohdl_ir::expr::Operator::BitAnd,
            Operator::BitOr => tohdl_ir::expr::Operator::BitOr,
            Operator::BitXor => tohdl_ir::expr::Operator::BitXor,
            _ => todo!("{}", format!("{:?}", op)),
        }
    }
}

impl Visitor for AstVisitor {
    fn visit_expr_call(&mut self, node: ExprCall) {
        {
            let value = node.func;
            self.visit_expr(*value);
        }
        let func_name = self.expr_stack.pop().unwrap();

        for value in node.args {
            self.visit_expr(value);
        }
        if node.keywords.len() > 0 {
            panic!("Keyword arguments are not supported {:?}", node.keywords);
        }

        // Parse arguments passed
        let args = self.expr_stack.iter().cloned().map(|x| match x {
            tohdl_ir::expr::Expr::Var(v) => v,
            _ => panic!("Unexpected expr in function args {:?}", x)
        }).collect();
        self.expr_stack.clear();

        // Create call node
        let call_node = tohdl_ir::graph::CallNode {
            args,
        };
        let call_node = self.graph.add_node(call_node);

        while let Some(prev) = self.node_stack.pop() {
            self.graph.add_edge(prev.node, call_node, prev.edge_type);
        }

        // Create external node
        let extern_node = tohdl_ir::graph::ExternalNode{name: func_name.to_string()};
        let extern_node = self.graph.add_node(extern_node);
        self.graph.add_edge(call_node, extern_node, NoneEdge.into());

        // Create func node
        let temp_var = tohdl_ir::expr::VarExpr::new(&format!("{}_0", func_name));
        let func_node = tohdl_ir::graph::FuncNode {
            params: vec![temp_var.clone()],
        };
        let func_node = self.graph.add_node(func_node);
        self.graph.add_edge(extern_node, func_node, NoneEdge.into());

        self.expr_stack.push(tohdl_ir::expr::Expr::Var(temp_var));
        self.node_stack.push((func_node, NoneEdge.into()).into());
    }
    fn visit_expr_tuple(&mut self, node: ExprTuple) {
        panic!("Tuples are unsupported {node:?}")
    }
    fn visit_arguments(&mut self, node: Arguments) {
        if let Some(FuncNode { params }) =
            FuncNode::concrete_mut(self.graph.get_node_mut(self.graph.get_entry()))
        {
            for arg in node.args {
                match arg {
                    ArgWithDefault {
                        range: _,
                        def,
                        default: _,
                    } => params.push(VarExpr::new(&def.arg.to_string())),
                }
            }
        } else {
            panic!("{node:?}")
        }
    }
    fn visit_stmt_function_def(&mut self, node: StmtFunctionDef) {
        self.graph.name = node.name.as_str().to_owned();
        self.generic_visit_stmt_function_def(node)
    }
    fn visit_stmt_aug_assign(&mut self, node: StmtAugAssign) {
        self.visit_expr(*node.target);
        let target = match self.expr_stack.pop().unwrap() {
            tohdl_ir::expr::Expr::Var(var) => var,
            _ => todo!(),
        };
        {
            let value = node.value;
            self.visit_expr(*value);
        }
        let value = self.expr_stack.pop().unwrap();
        let oper = AstVisitor::binop_mapping(node.op);
        let value = tohdl_ir::expr::Expr::BinOp(
            Box::new(tohdl_ir::expr::Expr::Var(target.clone())),
            oper,
            Box::new(value),
        );
        let node = tohdl_ir::graph::AssignNode {
            lvalue: target,
            rvalue: value,
        };
        let node = self.graph.add_node(node);

        while let Some(prev) = self.node_stack.pop() {
            self.graph.add_edge(prev.node, node, prev.edge_type);
        }
        self.node_stack.push((node, NoneEdge.into()).into());
    }
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
        let node = tohdl_ir::graph::AssignNode {
            lvalue: target,
            rvalue: value,
        };
        let node = self.graph.add_node(node);

        while let Some(prev) = self.node_stack.pop() {
            self.graph.add_edge(prev.node, node, prev.edge_type);
        }
        self.node_stack.push((node, NoneEdge.into()).into());
        self.print_debug_status();
    }
    fn visit_expr_bin_op(&mut self, node: ExprBinOp) {
        let oper = AstVisitor::binop_mapping(node.op);
        self.generic_visit_expr_bin_op(node);
        let right = self.expr_stack.pop().unwrap();
        let left = self.expr_stack.pop().unwrap();

        let expr = tohdl_ir::expr::Expr::BinOp(Box::new(left), oper, Box::new(right));
        self.expr_stack.push(expr);
    }
    fn visit_expr_compare(&mut self, node: ExprCompare) {
        let op = match node.ops[0] {
            CmpOp::Lt => tohdl_ir::expr::Operator::Lt,
            CmpOp::Gt => tohdl_ir::expr::Operator::Gt,
            CmpOp::LtE => tohdl_ir::expr::Operator::LtE,
            CmpOp::GtE => tohdl_ir::expr::Operator::GtE,
            CmpOp::Eq => tohdl_ir::expr::Operator::Eq,
            _ => todo!("{}", format!("{:?}", node.ops[0])),
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
        match node.value {
            Constant::Int(i) => {
                self.expr_stack
                    .push(tohdl_ir::expr::Expr::Int(tohdl_ir::expr::IntExpr::new(
                        str::parse::<i32>(&i.to_string()).unwrap(),
                    )))
            }
            Constant::Str(_) => {}
            _ => todo!("{node:?}"),
        };
    }
    fn visit_stmt_if(&mut self, node: StmtIf) {
        let prev = self.node_stack.pop().unwrap();
        {
            let value = node.test;
            self.visit_expr(*value);
        }
        let condition = self.expr_stack.pop().unwrap();
        // println!("condition {:?}", condition);
        self.print_debug_status();
        let ifelse = tohdl_ir::graph::BranchNode { cond: condition };
        let ifelse_node = self.graph.add_node(ifelse);
        self.node_stack
            .push((ifelse_node, BranchEdge::new(true).into()).into());

        for value in node.body {
            self.visit_stmt(value);
        }
        let true_final = self.node_stack.pop().unwrap();

        // println!("before orelse");
        self.print_debug_status();
        self.node_stack
            .push((ifelse_node, BranchEdge::new(false).into()).into());
        for value in node.orelse {
            self.visit_stmt(value);
        }
        let false_final = self.node_stack.pop().unwrap();

        self.graph.add_edge(prev.node, ifelse_node, prev.edge_type);

        self.node_stack.push(true_final);
        self.node_stack.push(false_final);

        // println!("post ifelse");
        self.print_debug_status();
    }
    fn visit_stmt_while(&mut self, node: StmtWhile) {
        let prev = self.node_stack.pop().unwrap();
        {
            let value = node.test;
            self.visit_expr(*value);
        }
        let condition = self.expr_stack.pop().unwrap();
        // println!("condition {:?}", condition);
        self.print_debug_status();
        let while_node = tohdl_ir::graph::BranchNode { cond: condition };
        let while_node = self.graph.add_node(while_node);
        self.node_stack
            .push((while_node, BranchEdge::new(true).into()).into());

        for value in node.body {
            self.visit_stmt(value);
        }
        let true_final = self.node_stack.pop().unwrap();

        self.graph.add_edge(prev.node, while_node, prev.edge_type);

        self.graph
            .add_edge(true_final.node, while_node, true_final.edge_type);

        self.node_stack
            .push((while_node, BranchEdge::new(false).into()).into());

        // println!("post while");
        self.print_debug_status();
    }
    fn visit_expr_yield(&mut self, node: ExprYield) {
        let mut prevs = self.node_stack.clone();
        self.node_stack.clear();
        if let Some(value) = node.value {
            self.visit_expr(*value);
        }
        let expr = self.expr_stack.pop().unwrap();
        let yield_node = tohdl_ir::graph::YieldNode { values: vec![expr] };
        let yield_node = self.graph.add_node(yield_node);
        while let Some(prev) = prevs.pop() {
            self.graph.add_edge(prev.node, yield_node, prev.edge_type);
        }
        self.node_stack.push((yield_node, NoneEdge.into()).into());
    }
    fn visit_stmt_return(&mut self, node: StmtReturn) {
        let prev = self.node_stack.pop().unwrap();
        if let Some(value) = node.value {
            self.visit_expr(*value);
        }
        let expr = self.expr_stack.pop().unwrap();
        let yield_node = tohdl_ir::graph::ReturnNode { values: vec![expr] };
        let yield_node = self.graph.add_node(yield_node);
        self.graph.add_edge(prev.node, yield_node, prev.edge_type);
        self.node_stack.push((yield_node, NoneEdge.into()).into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visitor() {
        let _python_source = r#"
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
    else:
        n = 1000
    j = n + 30
    while i < 100:
        i = i + 1
        j = j + 1
        yield i
    n = i + j
    return n
"#;
        let mut visitor = AstVisitor::default();
        let ast = ast::Suite::parse(python_source, "<embedded>").unwrap();

        // println!("ast {:#?}", ast);
        visitor.visit_stmt(ast[0].clone());

        let graph = visitor.get_graph();

        // println!("graph {}", graph.to_dot());
        graph.write_dot("visitor.dot")
    }

    #[test]
    fn aug_assign() {
        let python_source = r#"
def func(n):
    i = n + 10
    j = 10 + 15
    if j > 10:
        n = 100
        n = 150
    else:
        n = 1000
    j = n + 30
    while i < 100:
        i += 1
        j = j + 1
        yield i
    n = i + j
    return n
"#;
        let mut visitor = AstVisitor::default();
        let ast = ast::Suite::parse(python_source, "<embedded>").unwrap();

        // println!("ast {:#?}", ast);
        visitor.visit_stmt(ast[0].clone());

        let graph = visitor.get_graph();

        // println!("graph {}", graph.to_dot());
        graph.write_dot("visitor.dot")
    }
}
