use pyo3::prelude::*;
use tohdl_codegen::verilog::{
    graph_to_verilog, Context,
};

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

#[pyclass]
pub struct PyContext(Context);

#[pymethods]
impl PyContext {
    #[new]
    fn new(name: String) -> Self {
        Self(Context::new(name, vec![], Default::default()))
    }
}

#[pyfunction]
fn translate(code: &str) -> String {
    let visitor = tohdl_frontend::AstVisitor::from_text(code);
    let graph = visitor.get_graph();
    graph_to_verilog(graph)
}
