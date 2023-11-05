use super::edge::Edge;
use super::Node;
pub struct DiGraph(pub petgraph::Graph<Node, Edge>);

impl DiGraph {
    pub fn to_dot(&self) -> String {
        format!("{:?}", petgraph::dot::Dot::new(&self.0))
    }

    /// Gets node's data
    pub fn get_node(&self, node: usize) -> &Node {
        &self.0[petgraph::graph::NodeIndex::new(node)]
    }

    /// Iterates over pairs containing (node index, node)
    pub fn nodes(&self) -> impl Iterator<Item = usize> {
        self.0.node_indices().map(move |i| (i.index()))
    }

    /// Successors of a node
    pub fn succ(&self, node: usize) -> impl Iterator<Item = usize> + '_ {
        self.0
            .neighbors_directed(
                petgraph::graph::NodeIndex::new(node),
                petgraph::Direction::Outgoing,
            )
            .map(move |i| (i.index()))
    }

    /// Predecessors of a node
    pub fn preds(&self, node: usize) -> impl Iterator<Item = usize> + '_ {
        self.0
            .neighbors_directed(
                petgraph::graph::NodeIndex::new(node),
                petgraph::Direction::Incoming,
            )
            .map(move |i| (i.index()))
    }

    pub fn add_edge(&mut self, from: usize, to: usize, edge: Edge) {
        self.0.add_edge(
            petgraph::graph::NodeIndex::new(from),
            petgraph::graph::NodeIndex::new(to),
            edge,
        );
    }

    pub fn add_node(&mut self, node: Node) -> usize {
        self.0.add_node(node).index()
    }

    pub fn rmv_edge(&mut self, from: usize, to: usize) -> Edge {
        let edge_index = self
            .0
            .find_edge(
                petgraph::graph::NodeIndex::new(from),
                petgraph::graph::NodeIndex::new(to),
            )
            .unwrap();
        let edge_type = self.0.edge_weight(edge_index).unwrap().clone();
        self.0.remove_edge(edge_index);
        edge_type
    }
}

#[derive(Debug, Clone)]
pub struct Blank;
