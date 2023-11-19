use std::collections::hash_map::Entry;
use std::collections::HashMap;

use tohdl_ir::{expr::*, graph::*};

use crate::*;

#[derive(Default)]
pub struct RemoveUnreadVars {
    result: TransformResultType,
    pub(crate) var_to_definition: HashMap<VarExpr, NodeIndex>,
    pub(crate) var_to_ref_count: HashMap<VarExpr, usize>,
}

impl Transform for RemoveUnreadVars {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        self.work(graph);
        &self.result
    }
}

impl RemoveUnreadVars {
    pub(crate) fn make_reference_count(&mut self, graph: &CFG) {
        for idx in graph.nodes() {
            match graph.get_node(idx) {
                Node::Assign(AssignNode { lvalue, rvalue }) => {
                    self.var_to_definition.insert(lvalue.clone(), idx);
                    *self.var_to_ref_count.entry(lvalue.to_owned()).or_default() += 0;
                    for var in rvalue.get_vars() {
                        *self.var_to_ref_count.entry(var.to_owned()).or_default() += 1;
                    }
                }
                Node::Branch(BranchNode { cond }) => {
                    for var in cond.get_vars() {
                        *self.var_to_ref_count.entry(var.to_owned()).or_default() += 1;
                    }
                }
                Node::Call(CallNode { args }) => {
                    for var in args {
                        *self.var_to_ref_count.entry(var.to_owned()).or_default() += 1;
                    }
                }
                Node::Func(FuncNode { params }) => {
                    for var in params {
                        self.var_to_definition.insert(var.clone(), idx);
                        *self.var_to_ref_count.entry(var.to_owned()).or_default() += 0;
                    }
                }
                Node::Yield(TermNode { values }) | Node::Return(TermNode { values }) => {
                    for value in values {
                        for var in value.get_vars() {
                            *self.var_to_ref_count.entry(var.to_owned()).or_default() += 1;
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn remove_definition(&mut self, graph: &mut CFG, var: &VarExpr) {
        println!("removing {}", var);
        let idx = self.var_to_definition.get(var).unwrap();

        let mut node_to_remove = None;
        match graph.get_node_mut(*idx) {
            Node::Assign(AssignNode { lvalue, rvalue }) => {
                node_to_remove = Some(*idx);
                for var in rvalue.get_vars() {
                    *self.var_to_ref_count.entry(var).or_default() -= 1;
                }
            }
            Node::Func(FuncNode { params }) => {
                let index = params.iter().position(|v| v == var).unwrap();
                params.remove(index);
                for pred in graph.pred(*idx).collect::<Vec<NodeIndex>>() {
                    match graph.get_node_mut(pred) {
                        Node::Call(CallNode { args }) => {
                            let var = args.remove(index);
                            *self.var_to_ref_count.entry(var).or_default() -= 1;
                        }
                        _ => panic!(),
                    }
                }
            }
            _ => panic!(),
        }
        if let Some(idx) = node_to_remove {
            graph.rmv_node_and_reattach(idx);
        }
        self.var_to_ref_count.remove(var);
    }

    pub(crate) fn work(&mut self, graph: &mut CFG) {
        self.make_reference_count(graph);

        let mut to_be_removed = self
            .var_to_ref_count
            .iter()
            .filter(|&(_, v)| *v == 0)
            .map(|(k, _)| k.to_owned())
            .collect::<Vec<VarExpr>>();

        while let Some(var) = to_be_removed.pop() {
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
    use crate::{manager::PassManager, tests::*, transform::*, Transform};

    #[test]
    fn main() {
        let mut graph = make_even_fib();
        let mut manager = PassManager::default();

        manager.add_pass(InsertFuncNodes::transform);
        manager.add_pass(InsertCallNodes::transform);
        manager.add_pass(InsertPhi::transform);
        manager.add_pass(MakeSSA::transform);

        manager.apply(&mut graph);

        let mut pass = RemoveUnreadVars::default();
        pass.make_reference_count(&graph);
        println!("ref count {:?}", pass.var_to_ref_count);

        let mut pass = RemoveUnreadVars::default();
        pass.work(&mut graph);

        graph.write_dot("rmv_unread_vars.dot");
    }
}
