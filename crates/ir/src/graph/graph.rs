use std::collections::{HashMap, HashSet};

use petgraph::visit::IntoEdgeReferences;

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
pub struct DiGraph {
    pub graph: petgraph::stable_graph::StableDiGraph<Node, Edge>,
    pub entry: NodeIndex,
}

impl Default for DiGraph {
    fn default() -> Self {
        Self {
            graph: petgraph::stable_graph::StableDiGraph::default(),
            entry: 0.into(),
        }
    }
}

impl DiGraph {
    /// False positives and negatives are certainly possible
    pub fn graph_eq(a: &DiGraph, b: &DiGraph) -> bool {
        let a_root: NodeIndex = 0.into();
        let b_root: NodeIndex = 0.into();
        let a_nodes = a.dfs(a_root);
        let b_nodes = b.dfs(b_root);
        a_nodes.eq(&b_nodes)
    }

    pub fn to_dot(&self) -> String {
        struct NodeWithId<'a>(&'a Node, usize);

        impl std::fmt::Display for NodeWithId<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "({}) {}", self.1, self.0)
            }
        }

        // Create copy of graph with node indices
        let mut graph: petgraph::stable_graph::StableDiGraph<NodeWithId, Edge> =
            petgraph::stable_graph::StableDiGraph::new();

        let mut old_to_new: HashMap<NodeIndex, usize> = HashMap::new();
        for node in self.nodes() {
            let node_data = self.get_node(node);
            let new_idx = graph.add_node(NodeWithId(node_data, node.into()));
            old_to_new.insert(node, new_idx.index());
        }
        for node in self.nodes() {
            for succ in self.succ(node) {
                let edge = self.get_edge(node, succ).unwrap().clone();
                let new_from = old_to_new[&node];
                let new_to = old_to_new[&succ];
                graph.add_edge(
                    petgraph::graph::NodeIndex::new(new_from),
                    petgraph::graph::NodeIndex::new(new_to),
                    edge,
                );
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
        &self.graph[petgraph::graph::NodeIndex::new(node.into())]
    }

    pub fn get_node_mut(&mut self, node: NodeIndex) -> &mut Node {
        &mut self.graph[petgraph::graph::NodeIndex::new(node.into())]
    }

    pub fn get_edge(&self, from: NodeIndex, to: NodeIndex) -> Option<&Edge> {
        let edge_index = self
            .graph
            .find_edge(
                petgraph::graph::NodeIndex::new(from.into()),
                petgraph::graph::NodeIndex::new(to.into()),
            )
            .unwrap();
        self.graph.edge_weight(edge_index)
    }

    /// Iterates over pairs containing (node index, node)
    pub fn nodes(&self) -> impl Iterator<Item = NodeIndex> {
        self.graph
            .node_indices()
            .map(move |i| (i.index().into()))
            .collect::<Vec<NodeIndex>>()
            .into_iter()
    }

    /// Successors of a node
    pub fn succ(&self, node: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        self.graph
            .neighbors_directed(
                petgraph::graph::NodeIndex::new(node.into()),
                petgraph::Direction::Outgoing,
            )
            .map(move |i| (i.index().into()))
    }

    /// Predecessors of a node
    pub fn pred(&self, node: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        self.graph
            .neighbors_directed(
                petgraph::graph::NodeIndex::new(node.into()),
                petgraph::Direction::Incoming,
            )
            .map(move |i| (i.index().into()))
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, edge: Edge) {
        self.graph.add_edge(
            petgraph::graph::NodeIndex::new(from.into()),
            petgraph::graph::NodeIndex::new(to.into()),
            edge,
        );
    }

    /// Removes node and reattaches its predecessors to its successors
    pub fn rmv_node_and_reattach(&mut self, node: NodeIndex) {
        let preds = self.pred(node).collect::<Vec<_>>();
        let succs = self.succ(node).collect::<Vec<_>>();

        for pred in &preds {
            self.rmv_edge(*pred, node.clone());
        }
        for succ in &succs {
            self.rmv_edge(node.clone(), *succ);
        }
        for pred in &preds {
            for succ in &succs {
                self.add_edge(*pred, *succ, Edge::None);
            }
        }
        self.graph.remove_node(node.into());
    }

    pub fn add_node(&mut self, node: Node) -> NodeIndex {
        self.graph.add_node(node).index().into()
    }

    pub fn rmv_edge(&mut self, from: NodeIndex, to: NodeIndex) -> Edge {
        let edge_index = self
            .graph
            .find_edge(
                petgraph::graph::NodeIndex::new(from.into()),
                petgraph::graph::NodeIndex::new(to.into()),
            )
            .unwrap();
        let edge_type = self.graph.edge_weight(edge_index).unwrap().clone();
        self.graph.remove_edge(edge_index);
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

    #[test]
    fn test_reattach() {
        let mut graph = make_range();

        // graph.rmv_node_and_reattach(2.into());

        write_graph(&graph, "graph.dot");
    }
}
