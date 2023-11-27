use std::{collections::VecDeque, rc::Rc};

use tohdl_ir::{
    expr::VarExpr,
    graph::{AssignNode, BranchNode, CallNode, Edge, FuncNode, NodeIndex, NodeLike, TermNode, CFG},
};
use vast::v17::ast::{self as v, Sequential};

pub struct CaseFSM {
    graph: CFG,
    pub(crate) case: v::Case,
    var_stack: VecDeque<VarExpr>,
    ssa_separator: &'static str,
}

impl CaseFSM {
    fn new(graph: CFG) -> Self {
        CaseFSM {
            case: v::Case::new(v::Expr::Str("state".into())),
            graph,
            ssa_separator: ".",
            var_stack: VecDeque::new(),
        }
    }
    fn apply(&mut self) {
        let mut body = vec![];
        self.do_state(&mut body, self.graph.get_entry());
        let mut branch = v::CaseBranch::new("state0");
        branch.body = body;
        self.case.add_branch(branch);
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
            for succ in self.graph.succ(idx).collect::<Vec<_>>() {
                self.do_state(body, succ);
            }
        } else if let Some(node) = FuncNode::concrete_mut(node) {
            // Internal function (phi)
            for param in &node.params {
                let lhs = self.remove_separator(&param);
                let rhs = self
                    .var_stack
                    .pop_front()
                    .unwrap_or(VarExpr::new("error_pop"));
                body.push(v::Sequential::new_nonblk_assign(
                    v::Expr::new_ref(lhs.to_string()),
                    v::Expr::new_ref(rhs.to_string()),
                ));
            }
            for succ in self.graph.succ(idx).collect::<Vec<_>>() {
                self.do_state(body, succ);
            }
        } else if let Some(node) = CallNode::concrete_mut(node) {
            node.args = node
                .args
                .iter()
                .map(|arg| self.remove_separator(arg))
                .collect();
            if self.graph.succ(idx).collect::<Vec<NodeIndex>>().len() > 0 {
                // Internal func call
                for arg in &node.args {
                    self.var_stack.push_back(arg.clone());
                }
                for succ in self.graph.succ(idx).collect::<Vec<_>>() {
                    self.do_state(body, succ);
                }
            } else {
                // External func call
            }
        } else if let Some(node) = BranchNode::concrete_mut(node) {
            for var in node.cond.get_vars_iter_mut() {
                *var = self.remove_separator(var);
            }
            let mut ifelse = v::SequentialIfElse::new(v::Expr::new_ref(node.cond.to_string()));

            let mut succs = self.graph.succ(idx).collect::<Vec<_>>();
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

            ifelse.body = true_body;
            let temp_false = Sequential::If(v::SequentialIfElse::new(v::Expr::new_ref(&format!(
                "!{}",
                node.cond.to_string()
            ))));
            ifelse.else_branch = Some(Rc::new(temp_false));

            body.push(v::Sequential::If(ifelse));
        } else if let Some(node) = TermNode::concrete_mut(node) {
            for value in &mut node.values {
                for var in value.get_vars_iter_mut() {
                    *var = self.remove_separator(var);
                }
            }
            body.push(v::Sequential::new_nonblk_assign(
                v::Expr::new_ref("valid"),
                v::Expr::new_ref("1"),
            ));
            for (i, value) in node.values.iter().enumerate() {
                body.push(v::Sequential::new_nonblk_assign(
                    v::Expr::new_ref(&format!("out_{}", i)),
                    v::Expr::new_ref(value.to_string()),
                ));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use tohdl_passes::{
        manager::PassManager,
        transform::{BraunEtAl, InsertCallNodes, InsertFuncNodes},
        Transform,
    };

    use crate::tests::make_odd_fib;

    use super::*;
    #[test]
    fn demo() {
        let mut module = v::Module::new("foo");
        module.add_input("a", 32);
        let res = module.to_string();
        let exp = r#"module foo (
    input logic [31:0] a
);
endmodule
"#;
        assert_eq!(res, exp);
    }

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

        graph.write_dot("graph.dot");

        println!("original to subgraph {:?}", lower.node_to_subgraph);

        // Write all new subgraphs to files
        for (i, subgraph) in lower.get_subgraphs().iter().enumerate() {
            subgraph.write_dot(format!("lower_to_fsm_{}.dot", i).as_str());

            // let mut codegen = CodeGen::new(subgraph.clone(), i, lower.get_external_funcs(i));
            let mut codegen = CaseFSM::new(subgraph.clone());
            codegen.apply();
            println!("{}", codegen.case);
        }
    }
}
