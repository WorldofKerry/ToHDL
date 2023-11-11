pub mod parser;

#[cfg(test)]
mod tests {
    use super::*;
    use ast::*;
    use rustpython_parser::ast::Visitor;
    use rustpython_parser::{ast, Parse};
    use tohdl_ir::expr::Expr;

    #[test]
    fn it_works() {
        let python_source = r#"
def is_odd(i):
    return bool(i & 1)
"#;
        let ast = ast::Suite::parse(python_source, "<embedded>");

        assert!(ast.is_ok());

        // Print AST
        println!("{:#?}", ast.unwrap());
    }

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
        struct MyVisitor {
            expr_stack: Vec<tohdl_ir::expr::Expr>,
        }

        impl Default for MyVisitor {
            fn default() -> Self {
                Self { expr_stack: vec![] }
            }
        }

        impl Visitor for MyVisitor {
            fn visit_expr_bin_op(&mut self, node: ExprBinOp) {
                println!("visit_expr_bin_op {:?}", node);
                self.generic_visit_expr_bin_op(node);

                // Print expr stack
                println!("{:?}", self.expr_stack);
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

        let mut visitor = MyVisitor::default();
        let ast = ast::Suite::parse(python_source, "<embedded>").unwrap();

        // println!("{:#?}", ast);
        visitor.visit_stmt(ast[0].clone());
    }
}
