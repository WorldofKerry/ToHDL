mod elements;

use elements::*;
use pyo3::prelude::*;

#[pymodule]
fn testing2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Graph>()?;
    m.add_class::<Node>()?;
    Ok(())
}

#[pyclass(subclass)]
#[derive(Debug, Clone)]
struct Graph {
    graph: petgraph::Graph<Node, (), petgraph::Directed, u32>,
}

#[pymethods]
impl Graph {
    #[new]
    fn new() -> Self {
        Graph {
            graph: petgraph::Graph::new(),
        }
    }

    fn __str__(&self) -> String {
        format!("{:?}", self)
    }

    fn add_node(&mut self, node: Node) -> usize {
        self.graph.add_node(node).index()
    }

    fn add_edge(&mut self, node1: usize, node2: usize) -> usize {
        self.graph
            .add_edge(
                petgraph::graph::NodeIndex::new(node1),
                petgraph::graph::NodeIndex::new(node2),
                (),
            )
            .index()
    }
}
