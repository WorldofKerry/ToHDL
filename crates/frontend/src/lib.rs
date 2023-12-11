mod visitor;
pub use visitor::AstVisitor;

#[cfg(test)]
mod tests {

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
        // println!("{:#?}", ast.unwrap());
    }
}
