use tohdl_ir::graph::CFG;

fn code_to_graph(code: &str) -> CFG {
    tohdl_frontend::AstVisitor::from_text(code).get_graph()
}

pub fn aug_assign_str() -> &'static str {
    r#"
def aug_assign(a, b):
    a += 5
    return a
"#
}

pub fn aug_assign_graph() -> CFG {
    code_to_graph(aug_assign_str())
}

pub fn func_call_str() -> &'static str {
    r#"
def func_call(a):
    c = 3
    b = aug_assign(a, c)
    d = return_literal()
    return b + d
"#
}

pub fn func_call_graph() -> CFG {
    code_to_graph(func_call_str())
}

pub fn return_literal_str() -> &'static str {
    r#"
def return_literal():
    return 3
"#
}

pub fn return_literal_graph() -> CFG {
    code_to_graph(return_literal_str())
}
