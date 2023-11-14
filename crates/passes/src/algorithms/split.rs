use tohdl_ir::graph::DiGraph;
use tohdl_ir::graph::graph::NodeIndex;

/// Slices graph into a subgraph rooted at src
/// Inserts a call and func node what captures full context
/// Returns the new graph
pub fn split_graph(_graph: &mut DiGraph, _src: NodeIndex) -> DiGraph {
    let new_graph = DiGraph::default();
    new_graph
}

fn find_closure(_graph: &DiGraph, _src: NodeIndex) {
    // let mut closure = vec![];
}

#[cfg(test)]
mod tests {
    
    use crate::tests::make_fib;

    #[test]
    fn test_find_closure() {
        let _graph = make_fib();
    }
}

