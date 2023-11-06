use std::collections::HashSet;

use super::*;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

pub struct InsertPhi {}

impl InsertPhi {
    pub(crate) fn get_variables(&self, graph: &DiGraph) -> Vec<VarExpr> {
        let mut ret: Vec<VarExpr> = vec![];

        for node in graph.nodes() {
            match graph.get_node(node) {
                Node::Assign(AssignNode { ref lvalue, .. }) => {
                    if !ret.contains(lvalue) {
                        ret.push(lvalue.clone());
                    }
                }
                _ => {}
            }
        }

        ret
    }

    pub(crate) fn apply_to_var(&self, var: VarExpr, entry: usize, graph: &mut DiGraph) {
        let mut worklist: Vec<usize> = vec![];
        let mut ever_on_worklist: HashSet<usize> = HashSet::new();
        let mut already_has_phi: HashSet<usize> = HashSet::new();

        for node in graph.dfs(entry, &|_| true) {
            match graph.get_node(node) {
                Node::Assign(AssignNode { ref lvalue, .. }) => {
                    if lvalue == &var {
                        worklist.push(node);
                        ever_on_worklist.insert(node);
                    }
                }
                _ => {}
            }
        }

        ever_on_worklist.extend(worklist.clone());

        while let Some(node) = worklist.pop() {
            for d in self.dominance_frontier(graph, node) {
                if !already_has_phi.contains(&d) {
                    let d_data = graph.get_node_mut(d);
                    match d_data {
                        Node::Func(FuncNode { args }) => {
                            args.push(var.clone());
                        }
                        _ => {
                            panic!()
                        }
                    }
                    already_has_phi.insert(d);
                }
            }
        }
    }

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
    fn transform(&mut self, graph: &mut DiGraph) {
        for var in self.get_variables(graph) {
            self.apply_to_var(var, 0, graph);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use super::*;

    #[test]
    fn main() {
        let mut graph = make_range();

        insert_func::InsertFuncNodes {}.transform(&mut graph);
        insert_call::InsertCallNodes {}.transform(&mut graph);

        assert_eq!(InsertPhi {}.dominance_frontier(&graph, 2), vec![5]);

        assert_eq!(InsertPhi {}.get_variables(&graph), vec![VarExpr::new("i")]);

        let result = InsertPhi {}.apply_to_var(VarExpr::new("i"), 0, &mut graph);

        println!("result {:?}", result);

        write_graph(&graph, "insert_phi.dot");
    }
}
