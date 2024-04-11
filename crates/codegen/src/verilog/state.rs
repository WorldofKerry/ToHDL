use std::collections::{BTreeMap, VecDeque};

use tohdl_ir::{
    expr::VarExpr,
    graph::{
        AssignNode, BranchNode, CallNode, BranchEdge, FuncNode, Node, NodeIndex, ReturnNode, YieldNode,
        CFG,
    },
};
use vast::v05::ast::{self as v, Sequential};

use super::{
    memory::{LoadNode, NextStateNode, StoreNode},
    module::Context, expr::ToVerilog,
};

pub struct SingleStateLogic {
    name: usize,
    graph: CFG,
    pub(crate) body: Vec<Sequential>,
    var_stack: VecDeque<VarExpr>,
    ssa_separator: &'static str,
    external_funcs: BTreeMap<NodeIndex, usize>,
}

impl SingleStateLogic {
    pub fn new(graph: CFG, name: usize, external_funcs: BTreeMap<NodeIndex, usize>) -> Self {
        SingleStateLogic {
            body: vec![],
            graph,
            ssa_separator: ".",
            var_stack: VecDeque::new(),
            external_funcs,
            name,
        }
    }
    pub fn apply(&mut self, context: &mut Context) {
        let mut body = vec![];
        self.do_state(context, &mut body, self.graph.get_entry());
        self.body = body;
    }
    fn remove_separator(&self, var: &VarExpr) -> VarExpr {
        let raw = format!("{}", var);
        let processed = raw
            .split(self.ssa_separator)
            .collect::<Vec<&str>>()
            .join("");
        VarExpr::new(&processed)
    }
    fn do_state(&mut self, context: &mut Context, body: &mut Vec<Sequential>, idx: NodeIndex) {
        let node = &mut self.graph.get_node_mut(idx).clone();
        if let Some(node) = AssignNode::concrete_mut(node) {
            let lvalue = self.remove_separator(&node.lvalue);
            for var in node.rvalue.get_vars_iter_mut() {
                *var = self.remove_separator(var);
            }
            body.push(v::Sequential::new_nonblk_assign(
                v::Expr::new_ref(lvalue.to_string()),
                v::Expr::new_ref(node.rvalue.to_verilog()),
            ));
            for succ in self.graph.succs(idx).collect::<Vec<_>>() {
                self.do_state(context, body, succ);
            }
        } else if let Some(node) = LoadNode::concrete_mut(node) {
            let lvalue = self.remove_separator(&node.lvalue);
            for var in node.rvalue.get_vars_iter_mut() {
                *var = self.remove_separator(var);
            }
            body.push(v::Sequential::new_nonblk_assign(
                v::Expr::new_ref(lvalue.to_string()),
                v::Expr::new_ref(node.rvalue.to_verilog()),
            ));
            for succ in self.graph.succs(idx).collect::<Vec<_>>() {
                self.do_state(context, body, succ);
            }
        } else if let Some(node) = StoreNode::concrete_mut(node) {
            let lvalue = self.remove_separator(&node.lvalue);
            for var in node.rvalue.get_vars_iter_mut() {
                *var = self.remove_separator(var);
            }
            body.push(v::Sequential::new_nonblk_assign(
                v::Expr::new_ref(lvalue.to_string()),
                v::Expr::new_ref(node.rvalue.to_verilog()),
            ));
            for succ in self.graph.succs(idx).collect::<Vec<_>>() {
                self.do_state(context, body, succ);
            }
        } else if let Some(node) = FuncNode::concrete_mut(node) {
            // Function head
            context.memories.count = std::cmp::max(context.memories.count, node.params.len());
            for (i, param) in node.params.iter().enumerate() {
                let lhs = self.remove_separator(param);
                body.push(v::Sequential::new_nonblk_assign(
                    v::Expr::new_ref(lhs.to_string()),
                    v::Expr::new_ref(&format!("{}{}", context.memories.prefix, i)),
                ));
            }
            for succ in self.graph.succs(idx).collect::<Vec<_>>() {
                self.do_state(context, body, succ);
            }
        } else if let Some(node) = CallNode::concrete_mut(node) {
            node.args = node
                .args
                .iter()
                .map(|arg| self.remove_separator(arg))
                .collect();
            if !self.graph.succs(idx).collect::<Vec<NodeIndex>>().is_empty() {
                // Internal func call
                for arg in &node.args {
                    self.var_stack.push_back(arg.clone());
                }
                for succ in self.graph.succs(idx).collect::<Vec<_>>() {
                    self.do_state(context, body, succ);
                }
            }
        } else if let Some(node) = BranchNode::concrete_mut(node) {
            for var in node.cond.get_vars_iter_mut() {
                *var = self.remove_separator(var);
            }
            let mut ifelse = v::SequentialIfElse::new(v::Expr::new_ref(node.cond.to_verilog()));

            let mut succs = self.graph.succs(idx).collect::<Vec<_>>();
            if succs.len() != 2 {
                self.graph.write_dot("error.dot")
            }
            assert_eq!(succs.len(), 2, "Index: {idx}");

            // reorder so that the true branch is first
            if let Some(BranchEdge {condition}) = self.graph.get_edge(idx, succs[0]).unwrap().downcast_ref() {
                if !condition {
                    succs.swap(0, 1);
                }
            }

            let mut true_body = vec![];
            self.do_state(context, &mut true_body, succs[0]);
            let mut else_body = vec![];
            self.do_state(context, &mut else_body, succs[1]);

            ifelse.body = true_body;
            let mut temp_false = v::SequentialIfElse::default();
            temp_false.body = else_body;
            ifelse.set_else(temp_false);

            body.push(v::Sequential::IfElse(ifelse));
        } else if let Some(node) = YieldNode::concrete_mut(node) {
            for value in &mut node.values {
                for var in value.get_vars_iter_mut() {
                    *var = self.remove_separator(var);
                }
            }
            body.push(v::Sequential::new_nonblk_assign(
                v::Expr::new_ref(context.signals.valid.to_string()),
                v::Expr::Int(1),
            ));
            context.io.output_count = std::cmp::max(context.io.output_count, node.values.len());
            for (i, value) in node.values.iter().enumerate() {
                body.push(v::Sequential::new_nonblk_assign(
                    v::Expr::new_ref(&format!("{}{}", context.io.output_prefix, i)),
                    v::Expr::new_ref(value.to_verilog()),
                ));
            }
            for succ in self.graph.succs(idx).collect::<Vec<_>>() {
                self.do_state(context, body, succ);
            }
        } else if let Some(node) = ReturnNode::concrete_mut(node) {
            for value in &mut node.values {
                for var in value.get_vars_iter_mut() {
                    *var = self.remove_separator(var);
                }
            }
            context.io.output_count = std::cmp::max(context.io.output_count, node.values.len());
            for (i, value) in node.values.iter().enumerate() {
                body.push(v::Sequential::new_nonblk_assign(
                    v::Expr::new_ref(&format!("{}{}", context.io.output_prefix, i)),
                    v::Expr::new_ref(value.to_verilog()),
                ));
            }
            body.push(v::Sequential::new_nonblk_assign(
                v::Expr::new_ref(context.states.variable.to_string()),
                v::Expr::new_ref(context.states.done.to_string()),
            ));
            debug_assert_eq!(self.graph.succs(idx).collect::<Vec<_>>().len(), 0);
        } else if NextStateNode::downcastable(node) {
            body.push(v::Sequential::new_nonblk_assign(
                v::Expr::new_ref(context.states.variable.to_string()),
                v::Expr::new_ref(&format!(
                    "{}{}",
                    context.states.prefix, self.external_funcs[&idx]
                )),
            ));
        } else {
            panic!("Unexpected {}", node);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests::make_odd_fib;
    use tohdl_passes::{
        manager::PassManager,
        optimize::RemoveUnreadVars,
        transform::{
            BraunEtAl, ExplicitReturn, FixBranch, InsertCallNodes, InsertFuncNodes, Nonblocking,
        },
        Transform,
    };

    #[test]
    fn main() {
        let mut graph = make_odd_fib();

        let mut manager = PassManager::default();

        manager.add_pass(InsertFuncNodes::transform);
        manager.add_pass(InsertCallNodes::transform);
        manager.add_pass(BraunEtAl::transform);

        manager.apply(&mut graph);

        let mut lower = tohdl_passes::transform::LowerToFsm::default();
        lower.apply(&mut graph);

        // Write all new subgraphs to files
        let mut context = Context::default();
        for (i, subgraph) in lower.get_subgraphs().iter().enumerate() {
            let mut subgraph = subgraph.clone();
            crate::verilog::UseMemory::transform(&mut subgraph);
            Nonblocking::transform(&mut subgraph);
            RemoveUnreadVars::transform(&mut subgraph);
            FixBranch::transform(&mut subgraph);
            ExplicitReturn::transform(&mut subgraph);

            subgraph.write_dot(format!("debug_{}.dot", i).as_str());
            let mut codegen = SingleStateLogic::new(subgraph, i, lower.get_external_funcs(i));
            codegen.apply(&mut context);
        }
    }
}
