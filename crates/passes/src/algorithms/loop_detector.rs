use tohdl_ir::graph::{NodeIndex, CFG};

/// Field names based on
/// https://llvm.org/docs/LoopTerminology.html
#[derive(Debug)]
pub struct Loop {
    /// outside loop
    pub entering: Vec<NodeIndex>,
    pub exit: Vec<NodeIndex>,
    /// inside loop
    pub header: Vec<NodeIndex>,
    pub exiting: Vec<NodeIndex>,
    pub latches: Vec<NodeIndex>,
    pub members: Vec<NodeIndex>,
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
            let loop_parts = loopp.members.to_vec();

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

            // Disconnect outer loop
            for node in &loopp.header {
                for succ in subgraph.succs(*node).collect::<Vec<NodeIndex>>() {
                    subgraph.rmv_edge(*node, succ);
                }
            }

            // Work on subloops
            subloops.append(&mut detect_loops(&subgraph));
        }

        loops.append(&mut subloops)
    }

    loops
}

/// Does not explore nested loops
pub(crate) fn detect_loops(graph: &CFG) -> Vec<Loop> {
    let mut loops = vec![];

    let sccs = petgraph::algo::kosaraju_scc(&graph.graph);
    for scc in &sccs {
        if scc.len() <= 1 {
            continue;
        }

        // Find a node that has a pred not in ssc
        let mut entering = vec![];
        let mut header = vec![];
        for node in scc {
            let preds: Vec<NodeIndex> = graph.preds((*node).into()).collect();
            for pred in preds {
                if !scc.contains(&pred.into()) {
                    header.push((*node).into());
                    entering.push(pred);
                }
            }
        }
        // Find a node that has a succ not in ssc
        let mut exit = vec![];
        let mut exiting = vec![];
        for node in scc {
            let succs: Vec<NodeIndex> = graph.succs((*node).into()).collect();
            for succ in succs {
                if !scc.contains(&succ.into()) {
                    exit.push(succ);
                    exiting.push((*node).into());
                }
            }
        }
        // Find latches (preds of headers thare in scc)
        let mut latches = vec![];
        for node in &header {
            let preds: Vec<NodeIndex> = graph.preds(*node).collect();
            for pred in preds {
                if scc.contains(&pred.into()) {
                    latches.push(pred);
                }
            }
        }
        loops.push(Loop {
            entering,
            exit,
            header,
            exiting,
            latches,
            members: scc.iter().map(|n| (*n).into()).collect(),
        });
    }
    loops
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::tests::*;
    use crate::transform::*;
    use crate::BasicTransform;
    #[test]
    fn odd_fib() {
        let mut graph = make_even_fib();

        InsertFuncNodes::default().apply(&mut graph);
        InsertCallNodes::default().apply(&mut graph);
        InsertPhi::default().apply(&mut graph);
        MakeSSA::default().apply(&mut graph);
        // RemoveRedundantCalls::default().apply(&mut graph);

        let mut lower = LowerToFsm::default();
        lower.apply(&mut graph);

        graph.write_dot("loop_detector.dot");

        let loops = detect_loops(&graph);

        // println!("Loops {:#?}", loops);
    }

    #[test]
    fn double_while() {
        let mut graph = make_double_while();

        InsertFuncNodes::default().apply(&mut graph);
        InsertCallNodes::default().apply(&mut graph);
        BraunEtAl::transform(&mut graph);

        let mut lower = LowerToFsm::default();
        lower.apply(&mut graph);

        graph.write_dot("loop_detector.dot");

        let loops = detect_loops(&graph);

        // println!("Loops {:#?}", loops);

        let all_loops = detect_nested_loops(&graph);

        // println!("All loops {:#?}", all_loops);
    }
}
