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

    fn make_basic_branch() -> DiGraph<Element, ()> {
        let mut graph: DiGraph<Element, ()> = DiGraph::new();

        // prev = graph.add_node(AssignNode(i, Int(0)))
        // prev = ifelse = graph.add_node(BranchNode(BinOp(i, "<", n)), prev)

        // prev = postifelse = graph.add_node(AssignNode(n, i))
        // prev = graph.add_node(AssignNode(i, i), prev)
        // prev = graph.add_node(AssignNode(i, Int(10)), prev)

        // prev = graph.add_node(TrueNode(), ifelse)
        // prev = graph.add_node(AssignNode(i, i), prev)
        // prev = graph.add_node(AssignNode(i, i), prev)
        // prev = graph.add_node(AssignNode(i, Int(1)), prev)
        // prev = graph.add_node(AssignNode(i, Int(2)), prev)
        // prev = graph.add_node(AssignNode(i, i), prev)
        // prev = graph.add_node(AssignNode(i, i), prev)
        // prev = graph.add_node(AssignNode(i, Int(3)), prev, children=[postifelse])

        // prev = graph.add_node(FalseNode(), ifelse)
        // prev = graph.add_node(AssignNode(i, i), prev)
        // prev = graph.add_node(AssignNode(i, i), prev)
        // prev = graph.add_node(AssignNode(i, Int(1)), prev)
        // prev = graph.add_node(AssignNode(i, Int(2)), prev)
        // prev = graph.add_node(AssignNode(i, i), prev)
        // prev = graph.add_node(AssignNode(i, i), prev)
        // prev = graph.add_node(AssignNode(i, Int(4)), prev, children=[postifelse])

        // return graph

        let node0 = graph.add_node(Element::Assign(Assign {
            lvalue: "i".to_string(),
            rvalue: "0".to_string(),
            unique_id: "0".to_string(),
        }));

        let node1 = graph.add_node(Element::Branch(Branch {
            cond: "i < n".to_string(),
            unique_id: "1".to_string(),
        }));
        graph.add_edge(node0, node1, ());

        let node2 = graph.add_node(Element::Assign(Assign {
            lvalue: "n".to_string(),
            rvalue: "i".to_string(),
            unique_id: "2".to_string(),
        }));
        graph.add_edge(node1, node2, ());

        let node3 = graph.add_node(Element::Assign(Assign {
            lvalue: "i".to_string(),
            rvalue: "i".to_string(),
            unique_id: "3".to_string(),
        }));
        graph.add_edge(node2, node3, ());

        let node4 = graph.add_node(Element::Assign(Assign {
            lvalue: "i".to_string(),
            rvalue: "10".to_string(),
            unique_id: "4".to_string(),
        }));
        graph.add_edge(node3, node4, ());

        let node5 = graph.add_node(Element::BlockHead(BlockHead {
            phi: Default::default(),
            unique_id: "5".to_string(),
        }));
        graph.add_edge(node1, node5, ());

        let node6 = graph.add_node(Element::Assign(Assign {
            lvalue: "i".to_string(),
            rvalue: "i".to_string(),
            unique_id: "6".to_string(),
        }));
        graph.add_edge(node5, node6, ());

        let node7 = graph.add_node(Element::Assign(Assign {
            lvalue: "i".to_string(),
            rvalue: "i".to_string(),
            unique_id: "7".to_string(),
        }));
        graph.add_edge(node6, node7, ());

        let node8 = graph.add_node(Element::Assign(Assign {
            lvalue: "i".to_string(),
            rvalue: "1".to_string(),
            unique_id: "8".to_string(),
        }));

        let node9 = graph.add_node(Element::Assign(Assign {
            lvalue: "i".to_string(),
            rvalue: "2".to_string(),
            unique_id: "9".to_string(),
        }));

        graph
    }

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
        g.add_edge(node1, node2, ());
        g.add_edge(node2, node3, ());

        let children: Vec<_> = g
            .neighbors_directed(node2, petgraph::Direction::Outgoing)
            .collect();

        // Get Node with id 1
        let node = g.node_weight(NodeIndex::new(1)).unwrap();
        println!("{:?}", children);

        // Print the graph using Dot
        println!("{:?}", Dot::new(&g));

        // You can now perform various graph operations using the 'g' graph.
    }
}
