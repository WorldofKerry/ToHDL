use tohdl_ir::graph::NodeIndex;
use tohdl_ir::graph::CFG;

/// Slices graph into a subgraph rooted at src
/// Inserts a call and func node what captures full context
/// Returns the new graph
pub fn split_graph(_graph: &mut CFG, _src: NodeIndex) -> CFG {
    CFG::default()
}

fn find_closure(_graph: &CFG, _src: NodeIndex) {
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
