use std::collections::HashMap;

use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn testing2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<Graph>()?;
    Ok(())
}

/// Custom Class
#[pyclass(subclass)]
struct Graph {
    adj_list: HashMap<i32, Vec<i32>>,
    data: HashMap<i32, String>,
    counter: i32,
}

#[pymethods]
impl Graph {
    #[new]
    fn new() -> Self {
        Graph {
            adj_list: HashMap::new(),
            data: HashMap::new(),
            counter: 0,
        }
    }

    fn __str__(&self) -> String {
        serde_json::to_string(&self.adj_list).unwrap() + &serde_json::to_string(&self.data).unwrap()
    }

    fn add_node(&mut self, node: String) {
        self.counter += 1;
        self.data.insert(self.counter, node);
    }
}
