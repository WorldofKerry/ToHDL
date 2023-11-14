use std::collections::HashSet;

use crate::*;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

pub struct InsertPhi {
    result: TransformResultType,
}

impl Default for InsertPhi {
    fn default() -> Self {
        Self {
            result: TransformResultType::default(),
        }
    }
}

impl InsertPhi {
    /// Clears all args and params from all call and func nodes that have a predecessor
    pub(crate) fn clear_all_phis(&self, graph: &mut DiGraph) {
        for node in graph.nodes() {
            if graph.pred(node).count() == 0 {
                continue;
            }
            let node_data = graph.get_node_mut(node);
            match node_data {
                Node::Func(FuncNode { params: args }) => {
                    args.clear();
                }
                Node::Call(CallNode { args: params, .. }) => {
                    params.clear();
                }
                _ => {}
            }
        }
    }

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

    pub(crate) fn apply_to_var(&mut self, var: VarExpr, entry: NodeIndex, graph: &mut DiGraph) {
        let mut worklist: Vec<NodeIndex> = vec![];
        let mut ever_on_worklist: HashSet<NodeIndex> = HashSet::new();
        let mut already_has_phi: HashSet<NodeIndex> = HashSet::new();

        for node in graph.dfs(entry) {
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
                        Node::Func(FuncNode { params: args }) => {
                            self.result.did_work();
                            args.push(var.clone());
                        }
                        _ => {
                            panic!("join/merge at non-func node")
                        }
                    }

                    let preds = graph.pred(d).collect::<Vec<_>>();
                    for pred in preds {
                        match graph.get_node_mut(pred) {
                            Node::Call(CallNode {
                                args: ref mut params,
                                ..
                            }) => {
                                self.result.did_work();
                                params.push(var.clone());
                            }
                            _ => {
                                panic!("pred is not call node")
                            }
                        }
                    }
                    already_has_phi.insert(d);
                }
            }
        }
    }

    pub(crate) fn dominance_frontier(&self, graph: &DiGraph, node: NodeIndex) -> Vec<NodeIndex> {
        let mut ret: Vec<NodeIndex> = vec![];

        let n: NodeIndex = node;

        let dominance =
            petgraph::algo::dominators::simple_fast(&graph.graph, graph.get_entry().into());

        let zs = graph.nodes().collect::<Vec<_>>();
        let ms = graph.nodes().collect::<Vec<_>>();

        for z in &zs {
            for m in &ms {
                let m_succs = graph.succ(*m).collect::<Vec<_>>();
                let m_to_z = m_succs.contains(&z);

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
    fn apply(&mut self, graph: &mut DiGraph) -> &TransformResultType {
        self.clear_all_phis(graph);
        for var in self.get_variables(graph) {
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
            InsertPhi::default().get_variables(&graph),
            vec![VarExpr::new("i")]
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
