use core::panic;

use rustpython_parser::{ast, Parse};
use tohdl_ir::graph::DiGraph;

pub fn parse_func(text: &str) -> DiGraph {
    let ast = ast::Suite::parse(text, "<embedded>");

    println!("{:#?}", ast.as_ref().unwrap());

    let binding = ast.unwrap();
    let func = binding.get(0).unwrap();

    let body = match func {
        ast::Stmt::FunctionDef(ast::StmtFunctionDef { body, .. }) => body,
        _ => panic!("Not a function"),
    };

    let graph = DiGraph::new();

    for stmt in body {
        recurse_stmt(stmt);
    }

    graph
}

fn recurse_stmt(stmt: &ast::Stmt) {
    match stmt {
        ast::Stmt::Assign(ast::StmtAssign {
            range,
            targets,
            value,
            type_comment,
        }) => {
            println!("Assign {:?} = {:?}", targets, value);
        }
        _ => {}
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
        let ast = parse_func(python_source);
    }
}
