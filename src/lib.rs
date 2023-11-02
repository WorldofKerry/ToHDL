mod elements;

use elements::*;
use pyo3::prelude::*;

#[pymodule]
fn testing2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Graph>()?;
    Ok(())
}

#[pyclass(subclass)]
#[derive(Debug, Clone)]
struct Graph {
    graph: petgraph::Graph<PyObjectWrapper, (), petgraph::Directed, u32>,
}

#[pymethods]
impl Graph {
    #[new]
    fn new() -> Self {
        Graph {
            graph: petgraph::Graph::new(),
        }
    }

    fn add_node(&mut self, node: PyObject) -> usize {
        self.graph.add_node(PyObjectWrapper(node)).index()
    }

    fn add_edge(&mut self, source: usize, target: usize) -> usize {
        self.graph
            .add_edge(
                petgraph::graph::NodeIndex::new(source),
                petgraph::graph::NodeIndex::new(target),
                (),
            )
            .index()
    }

    fn rmv_node(&mut self, node: usize) {
        self.graph
            .remove_node(petgraph::graph::NodeIndex::new(node));
    }

    fn rmv_edge(&mut self, edge: usize) {
        self.graph
            .remove_edge(petgraph::graph::EdgeIndex::new(edge));
    }

    fn to_dot(&self) -> String {
        format!(
            "{:?}",
            petgraph::dot::Dot::with_config(&self.graph, &[petgraph::dot::Config::EdgeNoLabel])
        )
    }
}
