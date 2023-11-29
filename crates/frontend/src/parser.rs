use core::panic;

use rustpython_parser::ast;
use tohdl_ir::{expr::*, graph::*};

type ReturnType = (Node, Vec<Node>);

struct Parser {
    graph: CFG,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            graph: CFG::default(),
        }
    }

    pub fn parse_func(&self, text: &str) -> CFG {
        let ast = <ast::Suite as rustpython_parser::Parse>::parse(text, "<embedded>");

        println!("{:#?}", ast.as_ref().unwrap());

        let binding = ast.unwrap();
        let func = binding.get(0).unwrap();

        let body = match func {
            ast::Stmt::FunctionDef(ast::StmtFunctionDef { body, .. }) => body,
            _ => panic!("Not a function"),
        };

        let graph = CFG::default();

        for stmt in body {
            let result = self.parse_stmt(stmt);
            println!("{:?}", result);
        }

        graph
    }

    fn parse_stmt(&self, stmt: &ast::Stmt) -> ReturnType {
        match stmt {
            ast::Stmt::Assign(ast::StmtAssign {
                range: _,
                targets,
                value,
                type_comment: _,
            }) => {
                println!("Assign {:?} = {:?}", targets, value);
                let name = match targets.get(0).unwrap() {
                    ast::Expr::Name(ast::ExprName {
                        range: _,
                        id,
                        ctx: _,
                    }) => id.as_str(),
                    _ => panic!(),
                };

                println!("name {}", name);

                let node = Node::Assign(AssignNode {
                    lvalue: VarExpr::new(name),
                    rvalue: Expr::Int(IntExpr::new(0)),
                });
                return (node.clone(), vec![node]);
            }
            _ => {}
        }
        (
            Node::Assign(AssignNode {
                lvalue: VarExpr::new("i"),
                rvalue: Expr::Int(IntExpr::new(0)),
            }),
            vec![],
        )
    }

    fn parse_expr(&self, _expr: &ast::Expr) -> ReturnType {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let python_source = r#"
def is_odd(n):
    i = 0
    if i < n:
        i = i + 1
        yield i
    return
"#;
        let parser = Parser::new();
        let _ast = parser.parse_func(python_source);
    }
}
