use tohdl_ir::graph::CFG;

pub fn aug_assign_str() -> &'static str {
    r#"
def aug_assign(a, b):
    a += 5
    return a
"#
}

pub fn aug_assign_graph() -> CFG {
    let code = aug_assign_str();
    let visitor = tohdl_frontend::AstVisitor::from_text(code);

    let graph = visitor.get_graph();

    graph
}

pub fn func_call_str() -> &'static str {
    r#"
def func_call(a):
    c = 3
    b = aug_assign(a, c)
    return b
"#
}

pub fn func_call_graph() -> CFG {
    let code = func_call_str();
    let visitor = tohdl_frontend::AstVisitor::from_text(code);

    let graph = visitor.get_graph();

    graph
}
