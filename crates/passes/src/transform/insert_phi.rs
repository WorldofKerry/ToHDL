use std::collections::BTreeSet;
use std::collections::HashSet;

use crate::*;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

#[derive(Default)]
pub struct InsertPhi {
    result: TransformResultType,
}

impl InsertPhi {
    /// Clears all args and params from all call and func nodes that have a predecessor
    pub(crate) fn clear_all_phis(&self, graph: &mut CFG) {
        for node in graph.nodes() {
            if graph.pred(node).count() == 0 {
                continue;
            }
            let node_data = graph.get_node_mut(node);
            match FuncNode::concrete_mut(node_data) {
                Some(FuncNode { params }) => {
                    params.clear();
                }
                None => {}
            }
            match CallNode::concrete_mut(node_data) {
                Some(CallNode { args }) => {
                    args.clear();
                }
                None => {}
            }
        }
    }

    /// Gets all variable definitions
    pub(crate) fn get_call_var_defs(&self, graph: &CFG) -> BTreeSet<VarExpr> {
        graph
            .nodes()
            .flat_map(|idx| graph.get_node(idx).wrote_vars())
            .map(|var| var.clone())
            .collect()
    }

    pub(crate) fn apply_to_var(&mut self, var: VarExpr, entry: NodeIndex, graph: &mut CFG) {
        let mut worklist: Vec<NodeIndex> = vec![];
        let mut ever_on_worklist: HashSet<NodeIndex> = HashSet::new();
        let mut already_has_phi: HashSet<NodeIndex> = HashSet::new();

        for node in graph.dfs(entry) {
            for inner_var in graph.get_node(node).wrote_vars() {
                if inner_var == &var {
                    worklist.push(node);
                    ever_on_worklist.insert(node);
                }
            }
        }

        ever_on_worklist.extend(worklist.clone());

        while let Some(node) = worklist.pop() {
            for d in self.dominance_frontier(graph, node) {
                if !already_has_phi.contains(&d) {
                    let d_data = graph.get_node_mut(d);
                    match FuncNode::concrete_mut(d_data) {
                        Some(FuncNode { params: args }) => {
                            self.result.did_work();
                            args.push(var.clone());
                        }
                        None => panic!("join/merge at non-func node"),
                    }

                    let preds = graph.pred(d).collect::<Vec<_>>();
                    for pred in preds {
                        match CallNode::concrete_mut(graph.get_node_mut(pred)) {
                            Some(CallNode {
                                args: ref mut params,
                                ..
                            }) => {
                                self.result.did_work();
                                params.push(var.clone());
                            }
                            None => panic!("pred is not call node"),
                        }
                    }
                    already_has_phi.insert(d);
                }
            }
        }
    }

    pub(crate) fn dominance_frontier(&self, graph: &CFG, node: NodeIndex) -> Vec<NodeIndex> {
        let mut ret: Vec<NodeIndex> = vec![];

        let n: NodeIndex = node;

        let dominance =
            petgraph::algo::dominators::simple_fast(&graph.graph, graph.get_entry().into());

        let zs = graph.nodes().collect::<Vec<_>>();
        let ms = graph.nodes().collect::<Vec<_>>();

        for z in &zs {
            for m in &ms {
                let m_succs = graph.succ(*m).collect::<Vec<_>>();
                let m_to_z = m_succs.contains(z);

                // println!("m={} z={} m_to_z={} {}", m, z, m_to_z, graph.to_dot());
                let m_doms = dominance
                    .dominators(petgraph::graph::NodeIndex::new((*m).into()))
                    .unwrap()
                    .collect::<Vec<_>>();
                let n_dom_m = m_doms.contains(&petgraph::graph::NodeIndex::new(n.into()));

                let z_sdoms = dominance
                    .strict_dominators(petgraph::graph::NodeIndex::new((*z).into()))
                    .unwrap()
                    .collect::<Vec<_>>();
                let n_sdom_z = z_sdoms.contains(&petgraph::graph::NodeIndex::new(n.into()));

                if m_to_z && n_dom_m && !n_sdom_z {
                    ret.push(*z);
                }
            }
        }

        ret
    }
}

impl Transform for InsertPhi {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        self.clear_all_phis(graph);
        for var in self.get_call_var_defs(graph) {
            self.apply_to_var(var, graph.get_entry(), graph);
        }
        &self.result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;
    use crate::transform::*;

    #[test]
    fn range() {
        let mut graph = make_range();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);

        assert_eq!(
            InsertPhi::default().dominance_frontier(&graph, 3.into()),
            vec![6.into()]
        );

        assert_eq!(
            InsertPhi::default().get_call_var_defs(&graph),
            BTreeSet::from([VarExpr::new("i"), VarExpr::new("n")])
        );

        let result = InsertPhi::default().apply_to_var(VarExpr::new("i"), 0.into(), &mut graph);

        println!("result {:?}", result);

        write_graph(&graph, "insert_phi.dot");
    }

    #[test]
    fn fib() {
        let mut graph = make_fib();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);

        // assert_eq!(InsertPhi {}.dominance_frontier(&graph, 2), vec![5]);

        // assert_eq!(InsertPhi {}.get_variables(&graph), vec![VarExpr::new("i")]);

        // let result = InsertPhidefault().apply_to_var(VarExpr::new("i"), 0, &mut graph);

        // println!("result {:?}", result);

        insert_phi::InsertPhi::default().apply(&mut graph);
        insert_phi::InsertPhi::default().apply(&mut graph);
        insert_phi::InsertPhi::default().apply(&mut graph);

        write_graph(&graph, "insert_phi.dot");
    }
}
