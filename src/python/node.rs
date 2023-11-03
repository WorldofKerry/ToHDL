use pyo3::prelude::*;

#[pyclass(subclass)]
#[derive(Clone)]
pub struct PyObjectWrapper(pub PyObject);

#[pymethods]
impl PyObjectWrapper {
    #[new]
    fn new(data: PyObject) -> Self {
        PyObjectWrapper(data)
    }

    fn __str__(&self) -> PyResult<String> {
        todo!()
    }
}

impl std::ops::Deref for PyObjectWrapper {
    type Target = PyObject;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for PyObjectWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string: PyResult<String> = Python::with_gil(|py| {
            // Create a new dictionary
            let dict = pyo3::types::PyDict::new(py);

            // Add a single key-value pair to the dictionary
            let key = "key"; // Example key (can be any PyObject)
            let value = &self; // Example value (can be any PyObject)

            // Convert Rust types to PyObjects
            let py_key = key.to_object(py);
            let py_value = value.to_object(py);

            // Insert the key-value pair into the dictionary
            dict.set_item(py_key, py_value)?;

            let result = py.eval("str(key)", Some(dict), None);

            Ok(result.unwrap().extract::<String>().unwrap())
        });
        write!(f, "{}", string.unwrap())
    }
}
