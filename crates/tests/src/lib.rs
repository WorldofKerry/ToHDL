use tohdl_ir::graph::CFG;

pub fn make_aug_assign() -> CFG {
    let code = r#"
def aug_assign(a, b):
    a += 5
    return a
"#;
    let visitor = tohdl_frontend::AstVisitor::from_text(code);

    let graph = visitor.get_graph();

    graph
}

pub fn make_func_call() -> CFG {
    let code = r#"
def func_call(a):
    c = 3
    b = func_call(a, c)
    return b
"#;
    let visitor = tohdl_frontend::AstVisitor::from_text(code);

    let graph = visitor.get_graph();

    graph
}
