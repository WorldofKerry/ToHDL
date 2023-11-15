use std::collections::{HashMap, VecDeque};

use tohdl_ir::{
    expr::{Expr, VarExpr},
    graph::{DiGraph, Edge, Node, NodeIndex},
};

pub struct CodeGen {
    code: String,
    indent: usize,
    graph: DiGraph,
    ssa_separator: &'static str,
    var_stack: VecDeque<VarExpr>,
    external_funcs: HashMap<NodeIndex, usize>,
    name: usize,
    is_initial_func: bool,
}

impl CodeGen {
    pub fn new(graph: DiGraph, name: usize, external_funcs: HashMap<NodeIndex, usize>) -> Self {
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
        match self.graph.get_node(idx).clone() {
            Node::Assign(mut node) => {
                let lvalue = self.remove_separator(&node.lvalue);
                for var in node.rvalue.get_vars_iter() {
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
            }
            Node::Func(mut node) => {
                // println!("debug: {}", node);
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
                    for param in node.params {
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
            }
            Node::Call(mut node) => {
                // println!("debug: {}", node);
                node.args = node
                    .args
                    .iter()
                    .map(|arg| self.remove_separator(arg))
                    .collect();
                if self.graph.succ(idx).collect::<Vec<NodeIndex>>().len() > 0 {
                    // Internal func call
                    for arg in node.args {
                        self.var_stack.push_back(arg);
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
            }
            Node::Branch(mut node) => {
                // println!("debug: {}", node);
                for var in node.cond.get_vars_iter() {
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
            }
            Node::Yield(mut node) => {
                // println!("debug: {}", node);
                // let mut values: Vec<Expr> = vec![];
                for value in &mut node.values {
                    for var in value.get_vars_iter() {
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
            Node::Return(mut node) => {
                // println!("debug: {}", node);
                // let mut values: Vec<Expr> = vec![];
                for value in &mut node.values {
                    for var in value.get_vars_iter() {
                        *var = self.remove_separator(var);
                    }
                }

                self.code.push_str(&format!(
                    "{}return {}\n",
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
            _ => {}
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

    pub fn make_range() -> DiGraph {
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
    fn range() {
        let mut graph = make_range();

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

    pub fn make_yields() -> DiGraph {
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
}
