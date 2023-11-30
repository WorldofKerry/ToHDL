use std::collections::{BTreeMap, VecDeque};

use tohdl_ir::{
    expr::VarExpr,
    graph::{
        AssignNode, BranchNode, CallNode, Edge, FuncNode, NodeIndex, NodeLike, ReturnNode,
        TermNode, YieldNode, CFG,
    },
};
use vast::v17::ast::{self as v, Sequential};

use super::{memory::MemoryNode, module::Context};

pub struct SingleStateLogic<'ctx> {
    name: usize,
    graph: CFG,
    pub(crate) body: Vec<Sequential>,
    var_stack: VecDeque<VarExpr>,
    ssa_separator: &'static str,
    external_funcs: BTreeMap<NodeIndex, usize>,
    is_initial_func: bool,
    context: &'ctx Context,
}

impl<'a> SingleStateLogic<'a> {
    pub fn new(
        graph: CFG,
        name: usize,
        external_funcs: BTreeMap<NodeIndex, usize>,
        context: &'a Context,
    ) -> Self {
        SingleStateLogic {
            body: vec![],
            graph,
            ssa_separator: ".",
            var_stack: VecDeque::new(),
            external_funcs,
            name,
            is_initial_func: true,
            context,
        }
    }
    pub fn apply(&mut self) {
        let mut body = vec![];
        self.do_state(&mut body, self.graph.get_entry());
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
    fn do_state(&mut self, body: &mut Vec<Sequential>, idx: NodeIndex) {
        let node = &mut self.graph.get_node_mut(idx).clone();
        if let Some(node) = AssignNode::concrete_mut(node) {
            let lvalue = self.remove_separator(&node.lvalue);
            for var in node.rvalue.get_vars_iter_mut() {
                *var = self.remove_separator(var);
            }
            body.push(v::Sequential::new_nonblk_assign(
                v::Expr::new_ref(lvalue.to_string()),
                v::Expr::new_ref(node.rvalue.to_string()),
            ));
            for succ in self.graph.succs(idx).collect::<Vec<_>>() {
                self.do_state(body, succ);
            }
        } else if let Some(node) = MemoryNode::concrete_mut(node) {
            let lvalue = self.remove_separator(&node.lvalue);
            for var in node.rvalue.get_vars_iter_mut() {
                *var = self.remove_separator(var);
            }
            body.push(v::Sequential::new_nonblk_assign(
                v::Expr::new_ref(lvalue.to_string()),
                v::Expr::new_ref(node.rvalue.to_string()),
            ));
            for succ in self.graph.succs(idx).collect::<Vec<_>>() {
                self.do_state(body, succ);
            }
        } else if let Some(node) = FuncNode::concrete_mut(node) {
            if self.is_initial_func {
                self.is_initial_func = false;
                // Function head
                for (i, param) in node.params.iter().enumerate() {
                    let lhs = self.remove_separator(param);
                    body.push(v::Sequential::new_nonblk_assign(
                        v::Expr::new_ref(lhs.to_string()),
                        v::Expr::new_ref(&format!("mem_{}", i)),
                    ));
                }
            } else {
                // Internal function (phi)
                for param in &node.params {
                    let lhs = self.remove_separator(param);
                    let rhs = self
                        .var_stack
                        .pop_front()
                        .unwrap_or(VarExpr::new("error_pop"));
                    body.push(v::Sequential::new_nonblk_assign(
                        v::Expr::new_ref(lhs.to_string()),
                        v::Expr::new_ref(rhs.to_string()),
                    ));
                }
            }
            for succ in self.graph.succs(idx).collect::<Vec<_>>() {
                self.do_state(body, succ);
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
                    self.do_state(body, succ);
                }
            } else {
                // External func call
                body.push(v::Sequential::new_nonblk_assign(
                    v::Expr::new_ref("state"),
                    v::Expr::new_ref(&format!("state{}", self.external_funcs.get(&idx).unwrap())),
                ));
                for (i, arg) in node.args.iter().enumerate() {
                    body.push(v::Sequential::new_nonblk_assign(
                        v::Expr::new_ref(&format!("mem_{}", i)),
                        v::Expr::new_ref(arg.to_string()),
                    ));
                }
            }
        } else if let Some(node) = BranchNode::concrete_mut(node) {
            for var in node.cond.get_vars_iter_mut() {
                *var = self.remove_separator(var);
            }
            let mut ifelse = v::SequentialIfElse::new(v::Expr::new_ref(node.cond.to_string()));

            let mut succs = self.graph.succs(idx).collect::<Vec<_>>();
            assert_eq!(succs.len(), 2);

            // reorder so that the true branch is first
            match self.graph.get_edge(idx, succs[0]).unwrap() {
                Edge::Branch(true) => {}
                Edge::Branch(false) => {
                    succs.swap(0, 1);
                }
                _ => unreachable!(),
            }

            let mut true_body = vec![];
            self.do_state(&mut true_body, succs[0]);
            let mut else_body = vec![];
            self.do_state(&mut else_body, succs[1]);

            dbg!(&else_body);
            ifelse.body = true_body;
            let mut temp_false =
                v::SequentialIfElse::new(v::Expr::new_ref(format!("!{}", node.cond)));
            temp_false.body = else_body;
            dbg!(&temp_false);
            ifelse.set_else(v::Sequential::If(temp_false));

            body.push(v::Sequential::If(ifelse));
        } else if let Some(node) = YieldNode::concrete_mut(node) {
            for value in &mut node.values {
                for var in value.get_vars_iter_mut() {
                    *var = self.remove_separator(var);
                }
            }
            body.push(v::Sequential::new_nonblk_assign(
                v::Expr::new_ref(self.context.signals.valid.to_string()),
                v::Expr::Int(1),
            ));
            for (i, value) in node.values.iter().enumerate() {
                body.push(v::Sequential::new_nonblk_assign(
                    v::Expr::new_ref(&format!("out_{}", i)),
                    v::Expr::new_ref(value.to_string()),
                ));
            }
            for succ in self.graph.succs(idx).collect::<Vec<_>>() {
                self.do_state(body, succ);
            }
        } else if let Some(node) = ReturnNode::concrete_mut(node) {
            for value in &mut node.values {
                for var in value.get_vars_iter_mut() {
                    *var = self.remove_separator(var);
                }
            }
            body.push(v::Sequential::new_nonblk_assign(
                v::Expr::new_ref(self.context.signals.valid.to_string()),
                v::Expr::Int(1),
            ));
            body.push(v::Sequential::new_nonblk_assign(
                v::Expr::new_ref(self.context.signals.done.to_string()),
                v::Expr::Int(1),
            ));
            for (i, value) in node.values.iter().enumerate() {
                body.push(v::Sequential::new_nonblk_assign(
                    v::Expr::new_ref(&format!("out_{}", i)),
                    v::Expr::new_ref(value.to_string()),
                ));
            }
            for succ in self.graph.succs(idx).collect::<Vec<_>>() {
                self.do_state(body, succ);
            }
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
        transform::{BraunEtAl, InsertCallNodes, InsertFuncNodes, Nonblocking},
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
        let context = Context::default();
        for (i, subgraph) in lower.get_subgraphs().iter().enumerate() {
            let mut subgraph = subgraph.clone();
            crate::verilog::UseMemory::transform(&mut subgraph);
            Nonblocking::transform(&mut subgraph);
            RemoveUnreadVars::transform(&mut subgraph);

            subgraph.write_dot(format!("debug_{}.dot", i).as_str());
            let mut codegen =
                SingleStateLogic::new(subgraph, i, lower.get_external_funcs(i), &context);
            codegen.apply();
        }
    }
}