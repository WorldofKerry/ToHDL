pub mod python;
mod verilog;

#[cfg(test)]
pub(crate) mod tests {
    use tohdl_ir::graph::CFG;
    use tohdl_passes::{manager::PassManager, optimize::*, transform::*, Transform};

    pub fn make_odd_fib() -> CFG {
        let code = r#"
def even_fib():
    i = 0
    a = 0
    b = 1
    while a < n:
        if a % 2:
            yield a
        temp = a + b
        a = b
        b = temp
        i = i + 1
    yield 123
"#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);

        let graph = visitor.get_graph();

        graph
    }

    pub fn make_yields() -> CFG {
        let code = r#"
def even_fib():
    yield n + 1
    yield n + 2
    yield n + 3
"#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);

        let graph = visitor.get_graph();

        graph
    }
}
