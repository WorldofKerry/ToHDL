use std::collections::BTreeMap;

use tohdl_ir::{expr::*, graph::*};

use crate::*;

#[derive(Default)]
pub struct RemoveUnreadVars {
    result: TransformResultType,
    pub(crate) var_to_definition: BTreeMap<VarExpr, NodeIndex>,
    pub(crate) var_to_ref_count: BTreeMap<VarExpr, usize>,
}

impl BasicTransform for RemoveUnreadVars {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        self.work(graph);
        &self.result
    }
}

impl RemoveUnreadVars {
    pub(crate) fn make_reference_count(&mut self, graph: &CFG) {
        for idx in graph.nodes() {
            for var in graph.get_node(idx).referenced_vars() {
                *self.var_to_ref_count.entry(var.to_owned()).or_default() += 1;
            }
            for var in graph.get_node(idx).declared_vars() {
                self.var_to_definition.insert(var.clone(), idx);
                if !self.var_to_ref_count.contains_key(var) {
                    self.var_to_ref_count.insert(var.clone(), 0);
                }
            }
        }
    }

    pub(crate) fn remove_definition(&mut self, graph: &mut CFG, var: &VarExpr) {
        let idx = self
            .var_to_definition
            .get(var)
            .expect(&format!("{:?} {:?}", var, self.var_to_definition));
        // println!("removing {} {}", var, idx);

        if !graph.nodes().collect::<Vec<_>>().contains(idx) {
            // println!("early return");
            return;
        }

        // Special case for func node, where it's call nodes should be removed too
        match FuncNode::concrete_mut(graph.get_node_mut(*idx)) {
            Some(FuncNode { params }) => {
                if let Some(index) = params.iter().position(|v| v == var) {
                    params.remove(index);
                    for pred in graph.preds(*idx).collect::<Vec<NodeIndex>>() {
                        match CallNode::concrete_mut(graph.get_node_mut(pred)) {
                            Some(CallNode { args }) => {
                                let var = args.remove(index);
                                *self.var_to_ref_count.entry(var).or_default() -= 1;
                            }
                            _ => panic!(),
                        }
                    }
                } else {
                    // println!("{} {:?}", var, params);
                }
            }
            _ => {
                if graph.get_node_mut(*idx).undefine_var(var) {
                    for referenced_var in graph.get_node(*idx).referenced_vars() {
                        *self
                            .var_to_ref_count
                            .entry(referenced_var.clone())
                            .or_default() -= 1;
                    }
                    graph.rmv_node_and_reattach(*idx);
                }
            }
        }
        self.var_to_ref_count.remove(var);
    }

    pub(crate) fn work(&mut self, graph: &mut CFG) {
        self.make_reference_count(graph);
        println!("{:?}", self.var_to_ref_count);
        graph.write_dot("RemoveUnreadVars");

        // dbg!(&self.var_to_ref_count);

        let mut to_be_removed = self
            .var_to_ref_count
            .iter()
            .filter(|&(_, v)| *v == 0)
            .map(|(k, _)| k.to_owned())
            .collect::<Vec<VarExpr>>();

        while let Some(var) = to_be_removed.pop() {
            if !self.var_to_definition.contains_key(&var) {
                continue;
            }
            self.remove_definition(graph, &var);
            to_be_removed.append(
                &mut self
                    .var_to_ref_count
                    .iter()
                    .filter(|&(_, v)| *v == 0)
                    .map(|(k, _)| k.to_owned())
                    .collect::<Vec<VarExpr>>(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{manager::PassManager, tests::*, transform::*, BasicTransform};

    #[test]
    fn even_fib() {
        let mut graph = make_even_fib();
        let mut manager = PassManager::default();

        manager.add_pass(InsertFuncNodes::transform);
        manager.add_pass(InsertCallNodes::transform);
        manager.add_pass(InsertPhi::transform);
        manager.add_pass(MakeSSA::transform);

        manager.apply(&mut graph);

        let mut pass = RemoveUnreadVars::default();
        pass.make_reference_count(&graph);
        // println!("ref count {:?}", pass.var_to_ref_count);

        let mut pass = RemoveUnreadVars::default();
        pass.work(&mut graph);

        graph.write_dot("rmv_unread_vars.dot");
    }
}
