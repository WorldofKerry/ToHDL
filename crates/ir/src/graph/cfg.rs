use std::collections::BTreeMap;

use petgraph::visit::{EdgeRef, IntoEdgeReferences};

use crate::expr::VarExpr;

use super::edge::Edge;
use super::{FuncNode, Node};

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
    pub name: String,
    pub graph: petgraph::stable_graph::StableDiGraph<Box<dyn Node>, Edge>,
    pub entry: NodeIndex,
}

impl Default for CFG {
    fn default() -> Self {
        Self {
            name: Default::default(),
            graph: petgraph::stable_graph::StableDiGraph::default(),
            entry: 0.into(),
        }
    }
}

impl CFG {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            graph: petgraph::stable_graph::StableDiGraph::default(),
            entry: 0.into(),
        }
    }
    pub fn set_entry(&mut self, entry: NodeIndex) {
        self.entry = entry;
    }

    pub fn get_entry(&self) -> NodeIndex {
        self.entry
    }

    pub fn get_inputs(&self) -> impl Iterator<Item = &VarExpr> {
        let idx = self.get_entry();
        if let Some(FuncNode { params }) = FuncNode::concrete(self.get_node(idx)) {
            params.iter()
        } else {
            panic!();
        }
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
        struct NodeWithId<'a> {
            data: &'a Box<dyn Node>,
            idx: usize,
            is_entry: bool,
        }

        impl std::fmt::Display for NodeWithId<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if self.is_entry {
                    write!(f, "Entry ({}) {}", self.idx, self.data)
                } else {
                    write!(f, "({}) {}", self.idx, self.data)
                }
            }
        }

        // Create copy of graph with node indices
        let mut graph: petgraph::stable_graph::StableDiGraph<NodeWithId, Edge> =
            petgraph::stable_graph::StableDiGraph::new();

        let mut old_to_new: BTreeMap<NodeIndex, usize> = BTreeMap::new();
        for idx in self.nodes() {
            let node_data = self.get_node(idx);
            let is_entry = idx == self.get_entry();
            let new_idx = graph.add_node(NodeWithId {
                data: node_data,
                idx: idx.into(),
                is_entry,
            });
            old_to_new.insert(idx, new_idx.index());
        }
        for node in self.nodes() {
            for succ in self.succs(node) {
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
    pub fn get_node(&self, idx: NodeIndex) -> &Box<dyn Node> {
        &self.graph[petgraph::graph::NodeIndex::new(idx.into())]
    }

    pub fn get_node_mut(&mut self, idx: NodeIndex) -> &mut Box<dyn Node> {
        &mut self.graph[petgraph::graph::NodeIndex::new(idx.into())]
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

    /// Iterates over edges
    pub fn edges<'a>(&'a self) -> impl Iterator<Item = (NodeIndex, NodeIndex)> + 'a {
        self.graph
            .edge_references()
            .map(|x| (x.source().into(), x.target().into()))
    }

    /// Successors of a node
    pub fn succs(&self, idx: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        self.graph
            .neighbors_directed(
                petgraph::graph::NodeIndex::new(idx.into()),
                petgraph::Direction::Outgoing,
            )
            .map(move |i| (i.index().into()))
    }

    /// Predecessors of a node
    pub fn preds(&self, idx: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        self.graph
            .neighbors_directed(
                petgraph::graph::NodeIndex::new(idx.into()),
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
    /// Fixes graph entry if removed node is graph entry
    pub fn rmv_node_and_reattach(&mut self, node: NodeIndex) {
        let preds = self.preds(node).collect::<Vec<_>>();
        let succs = self.succs(node).collect::<Vec<_>>();

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

        // Fix graph entry
        // if node == self.entry {
        //     assert_eq!(succs.len(), 1);
        //     self.entry = succs[0];
        // }
        if node == self.entry {
            if preds.len() > 0 {
                assert_eq!(preds.len(), 1);
                self.entry = preds[0];
            } else {
                assert_eq!(succs.len(), 1);
                self.entry = succs[0];
            }
        }
    }

    /// Removes node and all edges connected to it
    pub fn rmv_node(&mut self, node: NodeIndex) {
        let preds = self.preds(node).collect::<Vec<_>>();
        let succs = self.succs(node).collect::<Vec<_>>();

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
        T: Node,
    {
        self.graph.add_node(node.into()).index().into()
    }

    /// Inserts node after idx
    pub fn insert_node_after<T>(&mut self, node: T, idx: NodeIndex, edge_type: Edge) -> NodeIndex
    where
        T: Node,
    {
        let new = self.graph.add_node(node.into()).index().into();
        let succs = self.succs(idx).collect::<Vec<_>>();
        for succ in &succs {
            let edge_type = self.rmv_edge(idx, *succ);
            self.add_edge(new, *succ, edge_type);
        }
        self.add_edge(idx, new, edge_type);
        new
    }

    /// Inserts node before idx
    pub fn insert_node_before<T>(&mut self, node: T, idx: NodeIndex, edge_type: Edge) -> NodeIndex
    where
        T: Node,
    {
        let new = self.graph.add_node(node.into()).index().into();
        let preds = self.preds(idx).collect::<Vec<_>>();

        for pred in &preds {
            let edge_type = self.rmv_edge(*pred, idx);
            self.add_edge(*pred, new, edge_type);
        }

        self.add_edge(new, idx, edge_type);

        new
    }

    /// Adds successor to node
    pub fn insert_succ<T>(&mut self, node: T, idx: NodeIndex, edge_type: Edge) -> NodeIndex
    where
        T: Node,
    {
        let new = self.graph.add_node(node.into()).index().into();
        self.add_edge(idx, new, edge_type);
        new
    }

    pub fn replace_node<T>(&mut self, idx: NodeIndex, node: T)
    where
        T: Node,
    {
        *self.graph.node_weight_mut(idx.into()).unwrap() = Box::new(node);
    }

    pub fn add_node_boxed(&mut self, node: Box<dyn Node>) -> NodeIndex {
        self.graph.add_node(node).index().into()
    }

    pub fn rmv_edge(&mut self, from: NodeIndex, to: NodeIndex) -> Edge {
        let edge_index = self
            .graph
            .find_edge(
                petgraph::graph::NodeIndex::new(from.into()),
                petgraph::graph::NodeIndex::new(to.into()),
            )
            .unwrap_or_else(|| panic!("Missing edge from {} to {}", from, to));
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

            for succ in self.succs(node) {
                stack.push(succ);
            }
        }

        visited
    }

    /// Finds all exit nodes of the graph
    pub fn find_exits<'a>(graph: &'a CFG) -> impl Iterator<Item = NodeIndex> + 'a {
        graph
            .nodes()
            .filter(|&x| !graph.succs(x).peekable().peek().is_some())
    }

    /// Merge other graph into graph
    pub fn merge_graph(graph: &mut CFG, other: &CFG) {
        let nodes = other.nodes().collect::<Vec<_>>();
        let edges = other.edges().collect::<Vec<_>>();
        let mut mapping = BTreeMap::<NodeIndex, NodeIndex>::new();
        for idx in nodes {
            let new_idx = graph.add_node_boxed(other.get_node(idx).clone());
            mapping.insert(idx, new_idx);
        }
        for (source, target) in edges {
            let edge = other.get_edge(source, target);
            match edge {
                Some(e) => graph.add_edge(
                    mapping.get(&source).unwrap().clone(),
                    mapping.get(&target).unwrap().clone(),
                    e.clone(),
                ),
                None => panic!("Edge not in edges {:?}", edge),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap};

    /// Test how hashtable implements its mutability
    /// e.g. iterate over elements and mutate them
    /// but not iterate over elements and add/remove elements
    #[test]
    fn goal() {
        let graph: HashMap<usize, i32> = HashMap::from([(0, 10), (123, 456)]);

        // Some sort of filter (e.g. node has more than x successors)
        let _keys = graph.keys().filter(|&i| *i > 10).collect::<Vec<_>>();

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

    #[test]
    fn stateful_interior_mutability() {
        // struct MyHashMap<'a, T> {
        //     map: HashMap<i32, i32>,
        //     keys: Option<Keys<'a, i32, i32>>,
        //     phantom_data: PhantomData<T>,
        // }

        // #[derive(Default)]
        // struct Locked;

        // #[derive(Default)]
        // struct Unlocked;

        // impl<T> Default for MyHashMap<'_, T> {
        //     fn default() -> Self {
        //         Self {
        //             map: Default::default(),
        //             keys: None,
        //             phantom_data: Default::default(),
        //         }
        //     }
        // }

        // impl Deref for MyHashMap<'_, Unlocked> {
        //     type Target = HashMap<i32, i32>;

        //     fn deref(&self) -> &Self::Target {
        //         &self.map
        //     }
        // }

        // impl DerefMut for MyHashMap<'_, Unlocked> {
        //     fn deref_mut(&mut self) -> &mut Self::Target {
        //         &mut self.map
        //     }
        // }

        // impl MyHashMap<'_, Unlocked> {
        //     fn lock(self) -> MyHashMap<'static, Locked> {
        //         MyHashMap::<Locked> {
        //             map: self.map,
        //             keys: Some(self.map.keys()),
        //             phantom_data: Default::default(),
        //         }
        //     }
        // }

        // impl MyHashMap<'_, Locked> {
        //     pub fn get_keys(&self) -> &Keys<i32, i32> {
        //         &self.keys.unwrap()
        //     }
        //     pub fn get_mut(&mut self, key: i32) -> Option<&mut i32> {
        //         self.map.get_mut(&key)
        //     }
        //     pub fn insert(&mut self, key: i32, value: i32) {
        //         self.map.insert(key, value);
        //     }
        // }

        // let mut map = MyHashMap::<Locked>::default();
        // map.insert(1, 10);
    }
}
