use tohdl_ir::graph::DiGraph;
use tohdl_ir::graph::graph::NodeIndex;

/// Slices graph into a subgraph rooted at src
/// Inserts a call and func node what captures full context
/// Returns the new graph
pub fn split_graph(graph: &mut DiGraph, src: NodeIndex) -> DiGraph {
    let mut new_graph = DiGraph::new();
    new_graph
}

fn find_closure(graph: &DiGraph, src: NodeIndex) {
    // let mut closure = vec![];
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::make_fib;

    #[test]
    fn test_find_closure() {
        let mut graph = make_fib();
    }
}

