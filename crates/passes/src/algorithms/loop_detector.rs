use tohdl_ir::graph::{NodeIndex, CFG};

#[derive(Debug, Default)]
pub struct Loop {
    pub entry: Vec<NodeIndex>,
    /// outside loop
    pub exit: Vec<NodeIndex>,
    /// outside loop
    pub body: Vec<NodeIndex>,
}

/// Detects nested loops
fn detect_nested_loops(graph: &CFG) -> Vec<Loop> {
    let mut loops = detect_loops(graph);

    let mut prev_length = std::usize::MAX;

    let length = loops.len();
    while prev_length != length {
        prev_length = loops.len();

        let mut subloops = vec![];

        for loopp in &loops {
            // construct subgraph with only nodes in the loop body
            let mut subgraph = graph.clone();

            // make a vec of all loop parts
            let loop_parts = loopp
                .body
                .iter()
                // .chain(loopp.entry.iter())
                // .chain(loopp.exit.iter())
                .cloned()
                .collect::<Vec<_>>();

            // Remove all nodes and edges not in the loop body
            let mut nodes_to_remove = vec![];
            for node in subgraph.nodes() {
                if !loop_parts.contains(&node) {
                    nodes_to_remove.push(node);
                }
            }
            for node in nodes_to_remove {
                subgraph.rmv_node(node);
            }

            // Work on subloops
            subloops.append(&mut detect_loops(&subgraph));
        }

        loops.append(&mut subloops)
    }

    loops
}

/// Does not explore nested loops
fn detect_loops(graph: &CFG) -> Vec<Loop> {
    let mut loops = vec![];

    let sccs = petgraph::algo::kosaraju_scc(&graph.graph);
    println!("sccs: {:#?}", sccs);
    for scc in &sccs {
        if scc.len() <= 1 {
            continue;
        }
        let mut loopp = Loop::default();
        // Find a node that has a pred not in ssc
        for node in scc {
            let preds: Vec<NodeIndex> = graph.pred((*node).into()).collect();
            for pred in preds {
                if !scc.contains(&pred.into()) {
                    loopp.entry.push((*node).into());
                }
            }
        }
        // Find a node that has a succ not in ssc
        for node in scc {
            let succs: Vec<NodeIndex> = graph.succ((*node).into()).collect();
            for succ in succs {
                if !scc.contains(&succ.into()) {
                    loopp.exit.push(succ);
                }
            }
        }
        loopp.body = scc.iter().map(|n| (*n).into()).collect();
        loops.push(loopp);
    }
    loops
}

#[cfg(test)]
mod tests {
    use tohdl_ir::graph::NodeIndex;

    use super::*;
    use crate::tests::*;
    use crate::transform::*;
    use crate::Transform;
    #[test]
    fn odd_fib() {
        let mut graph = make_odd_fib();

        InsertFuncNodes::default().apply(&mut graph);
        InsertCallNodes::default().apply(&mut graph);
        InsertPhi::default().apply(&mut graph);
        MakeSSA::default().apply(&mut graph);
        // RemoveRedundantCalls::default().apply(&mut graph);

        let mut lower = LowerToFsm::default();
        lower.apply(&mut graph);

        write_graph(&graph, "loop_detector.dot");

        let loops = detect_loops(&graph);

        println!("Loops {:#?}", loops);

        // // Write all new subgraphs to files
        // for (i, subgraph) in lower.subgraphs.iter().enumerate() {
        //     write_graph(&subgraph, format!("lower_to_fsm_{}.dot", i).as_str());
        // }
    }

    #[test]
    fn double_while() {
        let mut graph = make_double_while();

        InsertFuncNodes::default().apply(&mut graph);
        InsertCallNodes::default().apply(&mut graph);
        InsertPhi::default().apply(&mut graph);
        MakeSSA::default().apply(&mut graph);
        // RemoveRedundantCalls::default().apply(&mut graph);

        let mut lower = LowerToFsm::default();
        lower.apply(&mut graph);

        write_graph(&graph, "loop_detector.dot");

        let loops = detect_loops(&graph);

        println!("Loops {:#?}", loops);

        let all_loops = detect_nested_loops(&graph);

        println!("All loops {:#?}", all_loops);

        // // Write all new subgraphs to files
        // for (i, subgraph) in lower.subgraphs.iter().enumerate() {
        //     write_graph(&subgraph, format!("lower_to_fsm_{}.dot", i).as_str());
        // }
    }
}
