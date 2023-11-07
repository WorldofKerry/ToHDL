use super::edge::Edge;
use super::Node;
pub struct DiGraph(pub petgraph::Graph<Node, Edge>);

impl DiGraph {
    pub fn new() -> Self {
        Self(petgraph::Graph::new())
    }

    pub fn to_dot(&self) -> String {
        format!("{:?}", petgraph::dot::Dot::new(&self.0))
    }

    /// Gets node's data
    pub fn get_node(&self, node: usize) -> &Node {
        &self.0[petgraph::graph::NodeIndex::new(node)]
    }

    pub fn get_node_mut(&mut self, node: usize) -> &mut Node {
        &mut self.0[petgraph::graph::NodeIndex::new(node)]
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
    pub fn pred(&self, node: usize) -> impl Iterator<Item = usize> + '_ {
        self.0
            .neighbors_directed(
                petgraph::graph::NodeIndex::new(node),
                petgraph::Direction::Incoming,
            )
            .map(move |i| (i.index()))
    }

    // /// Descendants of a node
    // pub fn desc(&self, node: usize) -> impl Iterator<Item = usize> + '_ {
    //     let mut dfs = petgraph::visit::Dfs::new(&self.0, petgraph::graph::NodeIndex::new(node));
    //     std::iter::from_fn(move || dfs.next(&self.0).map(|i| i.index()))
    // }

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

    /// DFS subtree rooted at source, with a filter
    pub fn dfs(&self, source: usize) -> Vec<usize> {
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
        source: usize,
        filter: &dyn Fn(&Node) -> bool,
    ) -> Vec<usize> {
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
    pub fn descendants_leaves(&self, source: usize, filter: &dyn Fn(&Node) -> bool) -> Vec<usize> {
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

        assert_eq!(
            graph.dfs(1, &|node| match node {
                Node::Branch(_) => false,
                _ => true,
            }),
            vec![1, 2, 3, 4]
        );

        // println!("result {:?}", result);

        write_graph(&graph, "graph.dot");
    }
}
