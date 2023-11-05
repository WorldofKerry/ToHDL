use super::*;
use crate::ir::graph::*;

pub struct InsertPhi {}

impl InsertPhi {
    pub(crate) fn dominance_frontier(&self, graph: &DiGraph, node: usize) -> Vec<usize> {
        let mut ret: Vec<usize> = vec![];

        let n: usize = node;

        let dominance = petgraph::algo::dominators::simple_fast(&graph.0, 0.into());

        let zs = graph.nodes().collect::<Vec<_>>();
        let ms = graph.nodes().collect::<Vec<_>>();

        for z in &zs {
            for m in &ms {
                let m_succs = graph.succ(*m).collect::<Vec<_>>();
                let m_to_z = m_succs.contains(&z);

                let m_doms = dominance
                    .dominators(petgraph::graph::NodeIndex::new(*m))
                    .unwrap()
                    .collect::<Vec<_>>();
                let n_dom_m = m_doms.contains(&petgraph::graph::NodeIndex::new(n));

                let z_sdoms = dominance
                    .strict_dominators(petgraph::graph::NodeIndex::new(*z))
                    .unwrap()
                    .collect::<Vec<_>>();
                let n_sdom_z = z_sdoms.contains(&petgraph::graph::NodeIndex::new(n));

                if m_to_z && n_dom_m && !n_sdom_z {
                    ret.push(*z);
                    println!(
                        "add -> z={} m={} m_to_z={} n_dom_m={} n_sdom_z={}",
                        z, m, m_to_z, n_dom_m, n_sdom_z
                    );
                } else {
                    println!(
                        "z={} m={} m_to_z={} n_dom_m={} n_sdom_z={}",
                        z, m, m_to_z, n_dom_m, n_sdom_z
                    );
                }
            }
        }

        ret
    }
}

impl Transform for InsertPhi {
    fn transform(&self, graph: &mut DiGraph) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::tests::*;

    #[test]
    fn main() {
        let mut graph = make_range();

        insert_func::InsertFuncNodes {}.transform(&mut graph);
        insert_call::InsertCallNodes {}.transform(&mut graph);

        let result = InsertPhi {}.dominance_frontier(&mut graph, 2);

        println!("result {:?}", result);

        write_graph(&graph, "make_ssa.dot");
    }
}
