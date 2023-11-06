use core::panic;

use rustpython_parser::ast;
use tohdl_ir::{expr::*, graph::*};

type ReturnType = (Node, Vec<Node>);

struct Parser {
    graph: DiGraph,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
        }
    }

    pub fn parse_func(&self, text: &str) -> DiGraph {
        let ast = <ast::Suite as rustpython_parser::Parse>::parse(text, "<embedded>");

        println!("{:#?}", ast.as_ref().unwrap());

        let binding = ast.unwrap();
        let func = binding.get(0).unwrap();

        let body = match func {
            ast::Stmt::FunctionDef(ast::StmtFunctionDef { body, .. }) => body,
            _ => panic!("Not a function"),
        };

        let graph = DiGraph::new();

        for stmt in body {
            let result = self.parse_stmt(stmt);
            println!("{:?}", result);
        }

        graph
    }

    fn parse_stmt(&self, stmt: &ast::Stmt) -> ReturnType {
        match stmt {
            ast::Stmt::Assign(ast::StmtAssign {
                range,
                targets,
                value,
                type_comment,
            }) => {
                println!("Assign {:?} = {:?}", targets, value);
                let name = match targets.get(0).unwrap() {
                    ast::Expr::Name(ast::ExprName { range, id, ctx }) => id.as_str(),
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

    fn parse_expr(&self, expr: &ast::Expr) -> ReturnType {
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
        let ast = parser.parse_func(python_source);
    }
}
