use pyo3::prelude::*;

#[pyclass(subclass)]
#[derive(Clone, Debug)]
pub struct Node {
    pub data: PyObject,
}

#[pymethods]
impl Node {
    #[new]
    fn new(data: PyObject) -> Self {
        Node { data }
    }

    fn __str__(&self) -> String {
        format!("{:?}", self.data)
    }
}
