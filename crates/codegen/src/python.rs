use std::collections::{BTreeMap, VecDeque};

use tohdl_ir::{
    expr::{Expr, VarExpr},
    graph::*,
};

pub struct CodeGen {
    code: String,
    indent: usize,
    graph: CFG,
    ssa_separator: &'static str,
    var_stack: VecDeque<VarExpr>,
    external_funcs: BTreeMap<NodeIndex, usize>,
    name: usize,
    is_initial_func: bool,
}

impl CodeGen {
    pub fn new(graph: CFG, name: usize, external_funcs: BTreeMap<NodeIndex, usize>) -> Self {
        Self {
            code: String::new(),
            indent: 0,
            graph,
            ssa_separator: ".",
            var_stack: VecDeque::new(),
            external_funcs: external_funcs,
            name: name,
            is_initial_func: true,
        }
    }
    pub fn work(&mut self, idx: NodeIndex) {
        let node = &mut self.graph.get_node_mut(idx).clone();
        if let Some(node) = AssignNode::concrete_mut(node) {
            let lvalue = self.remove_separator(&node.lvalue);
            for var in node.rvalue.get_vars_iter_mut() {
                *var = self.remove_separator(var);
            }
            self.code.push_str(&format!(
                "{}{} = {}\n",
                " ".repeat(self.indent),
                lvalue,
                node.rvalue
            ));
            for succ in self.graph.succ(idx).collect::<Vec<_>>() {
                self.work(succ);
            }
        } else if let Some(node) = FuncNode::concrete_mut(node) {
            if self.is_initial_func {
                self.is_initial_func = false;
                // Function head
                node.params = node
                    .params
                    .iter()
                    .map(|arg| self.remove_separator(arg))
                    .collect();
                self.code.push_str(&format!(
                    "{}def func{}({}):\n",
                    " ".repeat(self.indent),
                    self.name,
                    node.params
                        .iter()
                        .map(|arg| format!("{}", arg))
                        .collect::<Vec<String>>()
                        .join(", ")
                ));
                self.indent += 4;
                for succ in self.graph.succ(idx).collect::<Vec<_>>() {
                    self.work(succ);
                }
                self.indent -= 4;
            } else {
                // Internal function (phi)
                for param in &node.params {
                    let param = self.remove_separator(&param);
                    self.code.push_str(&format!(
                        "{}{} = {}\n",
                        " ".repeat(self.indent),
                        param,
                        self.var_stack.pop_front().unwrap()
                    ));
                }
                for succ in self.graph.succ(idx).collect::<Vec<_>>() {
                    self.work(succ);
                }
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
                    self.work(succ);
                }
            } else {
                // External func call
                self.code.push_str(&format!(
                    "{}yield from func{}({})\n",
                    " ".repeat(self.indent),
                    self.external_funcs.get(&idx).unwrap(),
                    node.args
                        .iter()
                        .map(|arg| format!("{}", arg))
                        .collect::<Vec<String>>()
                        .join(", ")
                ));
            }
        } else if let Some(node) = BranchNode::concrete_mut(node) {
            // println!("debug: {}", node);
            for var in node.cond.get_vars_iter_mut() {
                *var = self.remove_separator(var);
            }
            self.code
                .push_str(&format!("{}if {}:\n", " ".repeat(self.indent), node.cond));
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

            self.indent += 4;
            self.work(succs[0]);
            self.indent -= 4;
            self.code
                .push_str(&format!("{}else:\n", " ".repeat(self.indent)));
            self.indent += 4;
            self.work(succs[1]);
            self.indent -= 4;
        } else if let Some(node) = TermNode::concrete_mut(node) {
            for value in &mut node.values {
                for var in value.get_vars_iter_mut() {
                    *var = self.remove_separator(var);
                }
            }

            self.code.push_str(&format!(
                "{}yield {}\n",
                " ".repeat(self.indent),
                node.values
                    .iter()
                    .map(|arg| format!("{}", arg))
                    .collect::<Vec<String>>()
                    .join(", ")
            ));
            for succ in self.graph.succ(idx).collect::<Vec<_>>() {
                self.work(succ);
            }
        }
    }
    pub fn get_code(&self) -> String {
        self.code.clone()
    }
    fn remove_separator(&self, var: &VarExpr) -> VarExpr {
        let raw = format!("{}", var);
        let processed = raw
            .split(self.ssa_separator)
            .collect::<Vec<&str>>()
            .join("");
        VarExpr::new(&processed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tohdl_passes::{manager::PassManager, optimize::*, transform::*, Transform};

    pub fn make_odd_fib() -> CFG {
        let code = r#"
def even_fib():
    i = 0
    a = 0
    b = 1
    while a < n:
        if a % 2:
            yield a
        temp = a + b
        a = b
        b = temp
        i = i + 1
    return 0
"#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);

        let graph = visitor.get_graph();

        graph
    }

    #[test]
    fn odd_fib() {
        let mut graph = make_odd_fib();

        let mut manager = PassManager::default();

        manager.add_pass(InsertFuncNodes::transform);
        manager.add_pass(InsertCallNodes::transform);
        manager.add_pass(InsertPhi::transform);
        manager.add_pass(MakeSSA::transform);
        manager.add_pass(RemoveUnreadVars::transform);
        // manager.add_pass(RemoveRedundantCalls::transform);

        manager.apply(&mut graph);

        let mut lower = tohdl_passes::transform::LowerToFsm::default();
        lower.apply(&mut graph);

        graph.write_dot("graph.dot");

        println!("original to subgraph {:?}", lower.node_to_subgraph);

        // Write all new subgraphs to files
        for (i, subgraph) in lower.get_subgraphs().iter().enumerate() {
            subgraph.write_dot(format!("lower_to_fsm_{}.dot", i).as_str());

            let mut codegen = CodeGen::new(subgraph.clone(), i, lower.get_external_funcs(i));
            codegen.work(subgraph.get_entry());
            let code = codegen.get_code();
            println!("{}", code);
        }
    }

    pub fn make_yields() -> CFG {
        let code = r#"
def even_fib():
    yield n + 1
    yield n + 2
    yield n + 3
"#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);

        let graph = visitor.get_graph();

        graph
    }

    #[test]
    fn yields() {
        let mut graph = make_yields();

        let mut manager = PassManager::default();

        manager.add_pass(InsertFuncNodes::transform);
        manager.add_pass(InsertCallNodes::transform);
        manager.add_pass(InsertPhi::transform);
        manager.add_pass(MakeSSA::transform);
        // manager.add_pass(RemoveRedundantCalls::transform);

        manager.apply(&mut graph);

        let mut lower = tohdl_passes::transform::LowerToFsm::default();
        lower.apply(&mut graph);

        graph.write_dot("graph.dot");

        println!("original to subgraph {:?}", lower.node_to_subgraph);

        // Write all new subgraphs to files
        for (i, subgraph) in lower.get_subgraphs().iter().enumerate() {
            subgraph.write_dot(format!("lower_to_fsm_{}.dot", i).as_str());

            let mut codegen = CodeGen::new(subgraph.clone(), i, lower.get_external_funcs(i));
            codegen.work(subgraph.get_entry());
            let code = codegen.get_code();
            println!("{}", code);
        }
    }

    pub fn make_branch() -> CFG {
        let code = r#"
def even_fib(n):
    a = 0
    if a > 1: 
        b = 10
    else:
        b = 11
        yield b
    yield a
    yield b
    if b % 10:
        yield a
        a = 15
    else:
        b = a + 2
    yield a
    yield b    
"#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);

        let graph = visitor.get_graph();

        graph
    }

    #[test]
    fn branch() {
        let mut graph = make_branch();

        let mut manager = PassManager::default();

        manager.add_pass(InsertFuncNodes::transform);
        manager.add_pass(InsertCallNodes::transform);
        manager.add_pass(InsertPhi::transform);
        manager.add_pass(MakeSSA::transform);
        manager.add_pass(RemoveUnreadVars::transform);
        // manager.add_pass(RemoveRedundantCalls::transform);

        manager.apply(&mut graph);

        let mut lower = tohdl_passes::transform::LowerToFsm::default();
        lower.apply(&mut graph);

        graph.write_dot("graph.dot");

        println!("original to subgraph {:?}", lower.node_to_subgraph);

        // Write all new subgraphs to files
        for (i, subgraph) in lower.get_subgraphs().iter().enumerate() {
            subgraph.write_dot(format!("lower_to_fsm_{}.dot", i).as_str());

            let mut codegen = CodeGen::new(subgraph.clone(), i, lower.get_external_funcs(i));
            codegen.work(subgraph.get_entry());
            let code = codegen.get_code();
            println!("{}", code);
        }
    }
}
