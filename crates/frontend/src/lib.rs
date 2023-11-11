pub mod parser;

#[cfg(test)]
mod tests {
    use super::*;
    use ast::StmtIf;
    // use crate::visitor::Visitor;
    use rustpython_parser::ast::Visitor;
    use rustpython_parser::{ast, Parse};

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
                yield i, j
            j = j + 1
        print()
        i = i + 1
"#;
        struct MyVisitor;
        impl Visitor for MyVisitor {
            fn visit_stmt_if(&mut self, node: StmtIf) {
                println!("{:?}", node);
                self.generic_visit_stmt_if(node)
            }
        }

        let mut visitor = MyVisitor;
        let ast = ast::Suite::parse(python_source, "<embedded>").unwrap();

        // println!("{:#?}", ast);
        visitor.visit_stmt(ast[0].clone());
    }
}
