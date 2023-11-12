use super::edge::Edge;
use super::Node;

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash, PartialOrd, Ord, Default)]
pub struct NodeIndex(pub usize);

impl std::fmt::Display for NodeIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "NodeIndex({})", self.0)
    }
}

impl From<usize> for NodeIndex {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl Into<usize> for NodeIndex {
    fn into(self) -> usize {
        self.0
    }
}

impl Into<petgraph::graph::NodeIndex> for NodeIndex {
    fn into(self) -> petgraph::graph::NodeIndex {
        petgraph::graph::NodeIndex::new(self.0)
    }
}

#[derive(Clone, Debug)]
pub struct DiGraph(pub petgraph::Graph<Node, Edge>);

impl PartialEq for DiGraph {
    fn eq(&self, other: &Self) -> bool {
        Self::graph_eq(&self.0, &other.0)
    }
}

impl DiGraph {
    pub fn new() -> Self {
        Self(petgraph::Graph::new())
    }

    fn graph_eq<N, E, Ty, Ix>(
        a: &petgraph::Graph<N, E, Ty, Ix>,
        b: &petgraph::Graph<N, E, Ty, Ix>,
    ) -> bool
    where
        N: PartialEq,
        E: PartialEq,
        Ty: petgraph::EdgeType,
        Ix: petgraph::graph::IndexType + PartialEq,
    {
        let a_ns = a.raw_nodes().iter().map(|n| &n.weight);
        let b_ns = b.raw_nodes().iter().map(|n| &n.weight);
        let a_es = a
            .raw_edges()
            .iter()
            .map(|e| (e.source(), e.target(), &e.weight));
        let b_es = b
            .raw_edges()
            .iter()
            .map(|e| (e.source(), e.target(), &e.weight));
        a_ns.eq(b_ns) && a_es.eq(b_es)
    }

    pub fn to_dot(&self) -> String {
        struct NodeWithId<'a>(&'a Node, usize);

        impl std::fmt::Display for NodeWithId<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "({}) {}", self.1, self.0)
            }
        }

        // Create copy of graph with node indices
        let mut graph: petgraph::Graph<NodeWithId, Edge> = petgraph::Graph::new();
        for node in self.0.node_indices() {
            let node_data = self.0.node_weight(node).unwrap();
            graph.add_node(NodeWithId(node_data, node.index()));
        }
        for node in self.0.node_indices() {
            for succ in self
                .0
                .neighbors_directed(node, petgraph::Direction::Outgoing)
            {
                let edge = self.0.find_edge(node, succ).unwrap();
                let edge_data = self.0.edge_weight(edge).unwrap();
                graph.add_edge(node.into(), succ.into(), edge_data.clone());
            }
        }
        format!("{}", petgraph::dot::Dot::new(&graph))
    }

    pub fn write_dot(&self, filename: &str) {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(filename).unwrap();
        file.write_all(self.to_dot().as_bytes()).unwrap();
    }

    /// Gets node's data
    pub fn get_node(&self, node: NodeIndex) -> &Node {
        &self.0[petgraph::graph::NodeIndex::new(node.into())]
    }

    pub fn get_node_mut(&mut self, node: NodeIndex) -> &mut Node {
        &mut self.0[petgraph::graph::NodeIndex::new(node.into())]
    }

    /// Iterates over pairs containing (node index, node)
    pub fn nodes(&self) -> impl Iterator<Item = NodeIndex> {
        self.0.node_indices().map(move |i| (i.index().into()))
    }

    /// Successors of a node
    pub fn succ(&self, node: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        self.0
            .neighbors_directed(
                petgraph::graph::NodeIndex::new(node.into()),
                petgraph::Direction::Outgoing,
            )
            .map(move |i| (i.index().into()))
    }

    /// Predecessors of a node
    pub fn pred(&self, node: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        self.0
            .neighbors_directed(
                petgraph::graph::NodeIndex::new(node.into()),
                petgraph::Direction::Incoming,
            )
            .map(move |i| (i.index().into()))
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, edge: Edge) {
        self.0.add_edge(
            petgraph::graph::NodeIndex::new(from.into()),
            petgraph::graph::NodeIndex::new(to.into()),
            edge,
        );
    }

    pub fn add_node(&mut self, node: Node) -> NodeIndex {
        self.0.add_node(node).index().into()
    }

    pub fn rmv_edge(&mut self, from: NodeIndex, to: NodeIndex) -> Edge {
        let edge_index = self
            .0
            .find_edge(
                petgraph::graph::NodeIndex::new(from.into()),
                petgraph::graph::NodeIndex::new(to.into()),
            )
            .unwrap();
        let edge_type = self.0.edge_weight(edge_index).unwrap().clone();
        self.0.remove_edge(edge_index);
        edge_type
    }

    /// DFS subtree rooted at source, with a filter
    pub fn dfs(&self, source: NodeIndex) -> Vec<NodeIndex> {
        let mut visited = vec![];
        let mut stack = vec![source];

        while let Some(node) = stack.pop() {
            if visited.contains(&node) {
                continue;
            }

            visited.push(node);

            for succ in self.succ(node) {
                stack.push(succ);
            }
        }

        visited
    }

    /// Get subtree excluding leaves rooted at source, with a filter
    pub fn descendants_internal(
        &self,
        source: NodeIndex,
        filter: &dyn Fn(&Node) -> bool,
    ) -> Vec<NodeIndex> {
        let mut stack = vec![source];
        let mut result = vec![];

        while let Some(node) = stack.pop() {
            let node_data = self.get_node(node);
            if filter(node_data) {
                result.push(node);

                for succ in self.succ(node) {
                    stack.push(succ);
                }
            }
        }

        result
    }

    /// Get leaves of subtree rooted at source, with a filter
    pub fn descendants_leaves(
        &self,
        source: NodeIndex,
        filter: &dyn Fn(&Node) -> bool,
    ) -> Vec<NodeIndex> {
        let mut stack = vec![source];
        let mut result = vec![];

        while let Some(node) = stack.pop() {
            let node_data = self.get_node(node);
            if filter(node_data) {
                result.push(node);
            } else {
                for succ in self.succ(node) {
                    stack.push(succ);
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    #[test]
    fn main() {
        let graph = make_range();

        let result = graph.descendants_internal(2.into(), &|node| match node {
            Node::Branch(_) => false,
            _ => true,
        });

        println!("result {:?}", result);

        write_graph(&graph, "graph.dot");
    }
}
