use tohdl_ir::{
    expr::VarExpr,
    graph::{DiGraph, NodeIndex},
};

pub struct CodeGen {
    code: String,
    indent: usize,
    graph: DiGraph,
    ssa_separator: &'static str,
}

impl CodeGen {
    pub fn new(graph: DiGraph) -> Self {
        Self {
            code: String::new(),
            indent: 0,
            graph,
            ssa_separator: ".",
        }
    }
    pub fn work(&mut self, idx: NodeIndex) {
        match self.graph.get_node(idx).clone() {
            tohdl_ir::graph::Node::Assign(mut node) => {
                let lvalue = self.remove_separator(&node.lvalue);
                let rvalue = node
                    .rvalue
                    .get_vars_iter()
                    .map(|var| self.remove_separator(var));
                self.code.push_str(&format!(
                    "{}{} = {}",
                    " ".repeat(self.indent),
                    lvalue,
                    rvalue
                        .map(|var| format!("{}", var))
                        .collect::<Vec<String>>()
                        .join(", ")
                ));
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

    pub fn make_odd_range() -> DiGraph {
        let code = r#"
def even_fib():
    i = 0
    while i < n:
        if i % 2:
            yield i
        i = i + 1
    return 0
"#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);

        let graph = visitor.get_graph();

        graph
    }

    #[test]
    fn range() {
        let mut graph = make_odd_range();

        let mut manager = PassManager::default();

        manager.add_pass(InsertFuncNodes::transform);
        manager.add_pass(InsertCallNodes::transform);
        manager.add_pass(InsertPhi::transform);
        manager.add_pass(MakeSSA::transform);
        manager.add_pass(RemoveRedundantCalls::transform);

        manager.apply(&mut graph);

        let mut lower = tohdl_passes::transform::LowerToFsm::default();
        lower.apply(&mut graph);

        graph.write_dot("graph.dot");

        println!("{:#?}", lower);

        // Write all new subgraphs to files
        for (i, subgraph) in lower.get_subgraphs().iter().enumerate() {
            subgraph.write_dot(format!("lower_to_fsm_{}.dot", i).as_str());
        }
    }
}
