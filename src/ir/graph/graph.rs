use super::edge::Edge;
use super::Node;
pub struct DiGraph(pub petgraph::Graph<Node, Edge>);

impl std::ops::Deref for DiGraph {
    type Target = petgraph::Graph<Node, Edge>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for DiGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DiGraph {
    pub fn to_dot(&self) -> String {
        format!("{:?}", petgraph::dot::Dot::new(&self.0))
    }

    /// Iterates over pairs containing (node index, node)
    pub fn nodes(&self) -> impl Iterator<Item = (usize, &Node)> {
        self.0.node_indices().map(move |i| (i.index(), &self.0[i]))
    }

    /// Successors of a node
    pub fn succ(&self, node: usize) -> impl Iterator<Item = (usize, &Node)> {
        self.0
            .neighbors_directed(
                petgraph::graph::NodeIndex::new(node),
                petgraph::Direction::Outgoing,
            )
            .map(move |i| (i.index(), &self.0[i]))
    }
}

#[derive(Debug, Clone)]
pub struct Blank;
