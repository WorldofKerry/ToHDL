use std::collections::BTreeMap;

use super::edge::Edge;
use super::{Node, NodeLike};

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

impl From<NodeIndex> for usize {
    fn from(val: NodeIndex) -> Self {
        val.0
    }
}

impl From<NodeIndex> for petgraph::graph::NodeIndex {
    fn from(val: NodeIndex) -> Self {
        petgraph::graph::NodeIndex::new(val.0)
    }
}

impl From<petgraph::graph::NodeIndex> for NodeIndex {
    fn from(val: petgraph::graph::NodeIndex) -> Self {
        Self(val.index())
    }
}

#[derive(Clone)]
pub struct CFG {
    pub graph: petgraph::stable_graph::StableDiGraph<Box<dyn NodeLike>, Edge>,
    pub entry: NodeIndex,
}

impl Default for CFG {
    fn default() -> Self {
        Self {
            graph: petgraph::stable_graph::StableDiGraph::default(),
            entry: 0.into(),
        }
    }
}

impl CFG {
    pub fn set_entry(&mut self, entry: NodeIndex) {
        self.entry = entry;
    }

    pub fn get_entry(&self) -> NodeIndex {
        self.entry
    }

    /// False positives and negatives are certainly possible
    pub fn graph_eq(a: &CFG, b: &CFG) -> bool {
        let a_root: NodeIndex = 0.into();
        let b_root: NodeIndex = 0.into();
        let a_nodes = a.dfs(a_root);
        let b_nodes = b.dfs(b_root);
        a_nodes.eq(&b_nodes)
    }

    pub fn to_dot(&self) -> String {
        struct NodeWithId<'a>(&'a Box<dyn NodeLike>, usize);

        impl std::fmt::Display for NodeWithId<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "({}) {}", self.1, self.0)
            }
        }

        // Create copy of graph with node indices
        let mut graph: petgraph::stable_graph::StableDiGraph<NodeWithId, Edge> =
            petgraph::stable_graph::StableDiGraph::new();

        let mut old_to_new: BTreeMap<NodeIndex, usize> = BTreeMap::new();
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
    pub fn get_node(&self, node: NodeIndex) -> &Box<dyn NodeLike> {
        &self.graph[petgraph::graph::NodeIndex::new(node.into())]
    }

    pub fn get_node_mut(&mut self, node: NodeIndex) -> &mut Box<dyn NodeLike> {
        &mut self.graph[petgraph::graph::NodeIndex::new(node.into())]
    }

    pub fn get_edge(&self, from: NodeIndex, to: NodeIndex) -> Option<&Edge> {
        let edge_index = self.graph.find_edge(
            petgraph::graph::NodeIndex::new(from.into()),
            petgraph::graph::NodeIndex::new(to.into()),
        )?;
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
            let edge_type = self.rmv_edge(*pred, node);
            for succ in &succs {
                self.add_edge(*pred, *succ, edge_type.clone());
            }
        }
        for succ in &succs {
            self.rmv_edge(node, *succ);
        }
        self.graph.remove_node(node.into());
    }

    /// Removes node and all edges connected to it
    pub fn rmv_node(&mut self, node: NodeIndex) {
        let preds = self.pred(node).collect::<Vec<_>>();
        let succs = self.succ(node).collect::<Vec<_>>();

        for pred in &preds {
            self.rmv_edge(*pred, node);
        }
        for succ in &succs {
            self.rmv_edge(node, *succ);
        }
        self.graph.remove_node(node.into());
    }

    pub fn add_node<T>(&mut self, node: T) -> NodeIndex
    where
        T: NodeLike,
    {
        self.graph.add_node(node.into()).index().into()
    }

    pub fn add_node_boxed(&mut self, node: Box<dyn NodeLike>) -> NodeIndex {
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

    // /// Get subtree excluding leaves rooted at source, with a filter
    // pub fn descendants_internal(
    //     &self,
    //     source: NodeIndex,
    //     filter: &dyn Fn(&Node) -> bool,
    // ) -> Vec<NodeIndex> {
    //     let mut stack = vec![source];
    //     let mut result = vec![];

    //     while let Some(node) = stack.pop() {
    //         let node_data = self.get_node(node);
    //         if filter(node_data) {
    //             result.push(node);

    //             for succ in self.succ(node) {
    //                 stack.push(succ);
    //             }
    //         }
    //     }

    //     result
    // }

    // /// Get leaves of subtree rooted at source, with a filter
    // pub fn descendants_leaves(
    //     &self,
    //     source: NodeIndex,
    //     filter: &dyn Fn(&Node) -> bool,
    // ) -> Vec<NodeIndex> {
    //     let mut stack = vec![source];
    //     let mut result = vec![];

    //     while let Some(node) = stack.pop() {
    //         let node_data = self.get_node(node);
    //         if filter(node_data) {
    //             result.push(node);
    //         } else {
    //             for succ in self.succ(node) {
    //                 stack.push(succ);
    //             }
    //         }
    //     }

    //     result
    // }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap};

    use super::*;
    use crate::tests::*;

    #[test]
    fn main() {
        // let graph = make_range();

        // let result = graph.descendants_internal(2.into(), &|node| match node {
        //     Node::Branch(_) => false,
        //     _ => true,
        // });

        // println!("result {:?}", result);

        // write_graph(&graph, "graph.dot");
    }

    #[test]
    fn test_reattach() {
        let graph = make_range();

        // graph.rmv_node_and_reattach(2.into());

        write_graph(&graph, "graph.dot");
    }

    /// Test how hashtable implements its mutability
    /// e.g. iterate over elements and mutate them
    /// but not iterate over elements and add/remove elements
    #[test]
    fn goal() {
        let mut graph: HashMap<usize, i32> = HashMap::from([(0, 10), (123, 456)]);

        // Some sort of filter (e.g. node has more than x successors)
        let keys = graph.keys().filter(|&i| *i > 10).collect::<Vec<_>>();

        fn transform(indexes: Vec<&usize>, graph: &mut HashMap<usize, i32>) {
            for index in indexes {
                if let Some(data) = graph.get_mut(index) {
                    *data = 999;
                } else {
                    panic!();
                }
            }
        }

        // Borrow error
        // transform(keys, &mut graph);
    }

    #[test]
    fn ref_cell_solution() {
        let graph: HashMap<usize, RefCell<i32>> =
            HashMap::from([(0, 10.into()), (123, 456.into())]);

        // Some sort of filter (e.g. node has more than x successors)
        let keys = graph.keys().filter(|&i| *i > 10).collect::<Vec<_>>();

        fn transform(indexes: Vec<&usize>, graph: &HashMap<usize, RefCell<i32>>) {
            for index in indexes {
                if let Some(data) = graph.get(index) {
                    *data.borrow_mut() = 999;
                } else {
                    panic!();
                }
            }
        }

        transform(keys, &graph);
        assert_eq!(graph, HashMap::from([(0, 10.into()), (123, 999.into())]))
    }

    #[test]
    fn elements_solution() {
        let mut graph: HashMap<usize, i32> = HashMap::from([(0, 10), (123, 456)]);

        // Some sort of filter (e.g. node has more than x successors)
        let elements = graph.iter_mut().filter(|(k, _)| *k > &10);

        fn transform<'a, T>(elements: T)
        where
            T: Iterator<Item = (&'a usize, &'a mut i32)>,
        {
            for (_, v) in elements {
                *v = 999;
            }
        }

        // Borrow error
        transform(elements);
        assert_eq!(graph, HashMap::from([(0, 10), (123, 999)]));
    }
}
