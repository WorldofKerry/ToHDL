use super::edge::Edge;
use super::Node;
pub struct Graph(pub petgraph::Graph<Node, Edge, petgraph::Directed, u32>);

impl std::ops::Deref for Graph {
    type Target = petgraph::Graph<Node, Edge, petgraph::Directed, u32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Graph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Graph {
    pub fn to_dot(&self) -> String {
        format!("{:?}", petgraph::dot::Dot::new(&self.0))
    }
}

#[derive(Debug, Clone)]
pub struct Blank;
