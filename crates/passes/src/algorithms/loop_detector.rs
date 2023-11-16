use tohdl_ir::graph::{NodeIndex, CFG};

#[derive(Debug, Default)]
pub struct Loop {
    pub entry: Vec<NodeIndex>,
    pub exit: Vec<NodeIndex>,
    pub body: Vec<NodeIndex>,
}

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
                    loopp.exit.push((*node).into());
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
}
