use std::collections::HashMap;
mod expr;

trait HasUniqueId {
    fn unique_id(&self) -> String;
}
#[derive(Debug)]
struct Assign {
    lvalue: String,
    rvalue: String,
    unique_id: String,
}

impl HasUniqueId for Assign {
    fn unique_id(&self) -> String {
        String::from("Assign")
    }
}
#[derive(Debug)]
struct Branch {
    cond: String,
    unique_id: String,
}
#[derive(Debug)]
struct BlockHead {
    phi: HashMap<String, String>,
    unique_id: String,
}

#[derive(Debug)]
enum Element {
    Assign(Assign),
    Branch(Branch),
    BlockHead(BlockHead),
}

#[cfg(test)]
mod tests {
    use crate::graph::{Assign, BlockHead, Branch, Element, HasUniqueId};
    use petgraph::algo::{dijkstra, min_spanning_tree};
    use petgraph::data::FromElements;
    use petgraph::dot::{Config, Dot};
    use petgraph::graph::{DiGraph, NodeIndex, UnGraph};

    #[test]
    fn test_petgraph() {
        let mut g: DiGraph<Element, ()> = DiGraph::new();

        // Create nodes and add them to the graph
        let assign1 = Assign {
            lvalue: "x".to_string(),
            rvalue: "10".to_string(),
            unique_id: "Lmao".to_string(),
        };
        let node1 = g.add_node(Element::Assign(assign1));

        let branch1 = Branch {
            cond: "y > 5".to_string(),
            unique_id: "Lmao".to_string(),
        };
        let node2 = g.add_node(Element::Branch(branch1));

        let blockHead1 = BlockHead {
            phi: Default::default(),
            unique_id: "Lmao".to_string(),
        };
        let node3 = g.add_node(Element::BlockHead(blockHead1));

        // Add edges if needed
        // g.add_edge(node1, node2, "Edge data here");

        // Print the graph using Dot
        println!("{:?}", Dot::new(&g));

        // You can now perform various graph operations using the 'g' graph.
    }
}
