use std::collections::BTreeMap;

use pyo3::prelude::*;
use tohdl_codegen::python::graph_to_python;
use tohdl_codegen::verilog::{graph_to_verilog, Context};

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
    m.add_function(wrap_pyfunction!(python_to_python_fsm, m)?)?;
    Ok(())
}

#[pyclass]
pub struct PyContext {
    pub main: String,
    pub functions: BTreeMap<String, String>,
}

#[pymethods]
impl PyContext {
    #[new]
    fn new(main: String, functions: BTreeMap<String, String>) -> Self {
        Self {main, functions}
    }
}

#[pyfunction]
fn translate(code: &str) -> String {
    let visitor = tohdl_frontend::AstVisitor::from_text(code);
    let graph = visitor.get_graph();
    graph_to_verilog(graph)
}

#[pyfunction]
fn python_to_python_fsm(code: &str) -> String {
    let visitor = tohdl_frontend::AstVisitor::from_text(code);
    let graph = visitor.get_graph();
    graph_to_python(graph)
}

mod tests {
    use super::*;

    #[test]
    pub fn floating_point_add() {
        let code = r#"
def floating_point_add(a_sign, a_exponent, a_mantissa, b_sign, b_exponent, b_mantissa):
    # Make sure a has larger by magnitude
    if a_exponent < b_exponent:
        temp_sign = a_sign
        a_sign = b_sign
        b_sign = temp_sign

        temp_exponent = a_exponent
        a_exponent = b_exponent
        b_exponent = temp_exponent

        temp_mantissa = a_mantissa
        a_mantissa = b_mantissa
        b_mantissa = temp_mantissa

    elif a_exponent == b_exponent:
        if a_mantissa < b_mantissa:
            temp_sign = a_sign
            a_sign = b_sign
            b_sign = temp_sign

            temp_exponent = a_exponent
            a_exponent = b_exponent
            b_exponent = temp_exponent

            temp_mantissa = a_mantissa
            a_mantissa = b_mantissa
            b_mantissa = temp_mantissa

    c_sign = a_sign

    # Add implicit one
    a_mantissa |= 1 << 23
    b_mantissa |= 1 << 23

    yield a_mantissa
    yield c_sign
    yield c_sign
        "#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);
        let graph = visitor.get_graph();
        let verilog = graph_to_verilog(graph);

        use std::fs::File;
        use std::io::Write;

        let mut file = File::create("output.sv").unwrap();
        // write!(file, "{}", verilog).unwrap();
    }
}
