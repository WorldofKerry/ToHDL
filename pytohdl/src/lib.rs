use pyo3::prelude::*;
use tohdl_codegen::python::CodeGen;
use tohdl_passes::{manager::PassManager, optimize::*, transform::*, Transform};

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn pytohdl(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(translate, m)?)?;
    Ok(())
}

#[pyfunction]
fn translate(code: &str) -> String {
    let visitor = tohdl_frontend::AstVisitor::from_text(code);
    let mut graph = visitor.get_graph();

    let mut manager = PassManager::default();
    manager.add_pass(InsertFuncNodes::transform);
    manager.add_pass(InsertCallNodes::transform);
    manager.add_pass(InsertPhi::transform);
    manager.add_pass(MakeSSA::transform);
    // manager.add_pass(RemoveRedundantCalls::transform);

    manager.apply(&mut graph);

    let mut lower = tohdl_passes::transform::LowerToFsm::default();
    lower.apply(&mut graph);

    // Write all new subgraphs to files
    let mut output = String::new();
    for (i, subgraph) in lower.get_subgraphs().iter().enumerate() {
        let mut codegen = CodeGen::new(subgraph.clone(), i, lower.get_external_funcs(i));
        codegen.work(subgraph.get_entry());
        let code = codegen.get_code();
        println!("{}", code);
        output.push_str(&code);
    }
    output
}
