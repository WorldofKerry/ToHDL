use std::collections::{BTreeMap, BTreeSet};

use crate::*;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

#[derive(Debug, Default)]
pub struct MakeSSA {
    visited: BTreeSet<NodeIndex>,
    var_counter: BTreeMap<VarExpr, usize>,
    stacks: BTreeMap<VarExpr, Vec<VarExpr>>,
    var_mapping: BTreeMap<VarExpr, VarExpr>,
    pub(crate) global_vars: Vec<VarExpr>,

    result: TransformResultType,
}

impl BasicTransform for MakeSSA {
    /// Applies transformation
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        self.rename(graph, graph.get_entry());

        // If a global var is not in initial func call, add it
        let node = graph.get_node_mut(graph.get_entry());
        // println!("makessa node {}", node);
        // println!("makessa global vars {:?}", self.global_vars);
        match FuncNode::concrete_mut(node) {
            Some(FuncNode { params }) => {
                for var in &self.global_vars {
                    if !params.contains(var) {
                        // println!("makessa pushing {}", var);
                        params.push(var.clone());
                    }
                }
            }
            _ => panic!(),
        }

        &self.result
    }
}

impl MakeSSA {
    /// View the arguments to a func broken at specific index
    pub fn test_rename(&mut self, graph: &mut CFG, node: NodeIndex) -> Vec<VarExpr> {
        // println!("test_rename starting at {}", node);

        self.rename(graph, node);

        // If a global var is not in initial func call, add it
        let node = graph.get_node_mut(graph.get_entry());
        // println!("makessa node {}", node);
        // println!("makessa global vars {:?}", self.global_vars);
        match FuncNode::concrete_mut(node) {
            Some(FuncNode { params }) => {
                for var in &self.global_vars {
                    if !params.contains(var) {
                        // println!("makessa pushing {}", var);
                        params.push(var.clone());
                    }
                }
            }
            _ => panic!(),
        }

        // println!("make_ssa global_vars {:?}", self.global_vars);

        // Map global vars to their names before ssa
        self.global_vars
            .iter()
            .map(|x| self.var_mapping.get(x).unwrap().clone())
            .collect()
    } // Make basic block subtree

    pub(crate) fn nodes_in_basic_block(&self, graph: &CFG, source: NodeIndex) -> Vec<NodeIndex> {
        let mut stack = vec![source];

        let mut result = vec![source];

        while let Some(node) = stack.pop() {
            let node_data = graph.get_node(node);
            let any = node_data.as_any();
            if any.is::<CallNode>() {
            } else if any.is::<BranchNode>() {
                if !result.contains(&node) {
                    result.push(node);
                }
            } else {
                if !result.contains(&node) {
                    result.push(node);
                }

                for succ in graph.succs(node) {
                    stack.push(succ);
                }
            }
        }
        result
    }

    pub(crate) fn special_descendants(&self, graph: &CFG, source: NodeIndex) -> Vec<NodeIndex> {
        let any = graph.get_node(source).as_any();
        let mut stack = if any.is::<CallNode>() || any.is::<FuncNode>() {
            graph.succs(source).collect::<Vec<NodeIndex>>()
        } else {
            vec![source]
        };

        // let mut stack = vec![source];
        let mut result = vec![];

        while let Some(node) = stack.pop() {
            let node_data = graph.get_node(node);
            let any = node_data.as_any();
            if any.is::<CallNode>() || any.is::<FuncNode>() {
                result.push(node);
            } else if any.is::<BranchNode>() {
                for succ in graph.succs(node) {
                    result.push(succ)
                }
            } else {
                for succ in graph.succs(node) {
                    stack.push(succ);
                }
            }
        }
        result
    }

    /// Generate new name
    pub(crate) fn gen_name(&mut self, var: &VarExpr) -> VarExpr {
        let count = *self.var_counter.get(var).unwrap_or(&0);

        self.var_counter.insert(var.clone(), count + 1);

        let name = format!("{}.{}", var.name, count);
        let new_var = VarExpr::new(&name);

        // Update var stack
        let stack = self.stacks.entry(var.clone()).or_default();
        stack.push(new_var.clone());

        // Update var mapping
        self.var_mapping.insert(new_var.clone(), var.clone());

        new_var
    }

    /// Assert read vars are apart of stacks, otherwise it is a global var
    fn update_global_vars_if_nessessary(&mut self, vars: &Vec<&VarExpr>) {
        for var in vars {
            // If mapped value is non-existant or empty
            // Then var must be a global var
            let mut flag = false;
            if let Some(stack) = self.stacks.get_mut(var) {
                if stack.is_empty() {
                    flag = true;
                }
            } else {
                flag = true;
            }

            if flag {
                let new = self.gen_name(var);
                self.global_vars.push(new);
            }
        }
    }

    /// Update LHS and RHS
    fn update_lhs_rhs(&mut self, node: &mut Box<dyn Node>) {
        if FuncNode::concrete(node).is_none() {
            self.update_global_vars_if_nessessary(&node.referenced_vars());
        }
        for var in node.referenced_exprs_mut() {
            let mapping = self.make_mapping();
            var.backwards_replace(&mapping);
        }
        for var in node.declared_vars_mut() {
            *var = self.gen_name(var);
        }
    }

    /// Converts variable stack to mapping, by taking last element
    fn make_mapping(&self) -> BTreeMap<VarExpr, Expr> {
        self.stacks
            .iter()
            .map(|(var, stack)| {
                (
                    var.clone(),
                    Expr::Var(stack.last().unwrap_or(&VarExpr::new("unused")).clone()),
                )
            })
            .collect()
    }

    fn rename(&mut self, graph: &mut CFG, node: NodeIndex) {
        // Check visited
        if self.visited.contains(&node) {
            return;
        }
        self.visited.insert(node);

        // println!(
        //     "rename node {}, basic block {:?}, special desc {:?}",
        //     node,
        //     self.nodes_in_basic_block(graph, node),
        //     self.special_descendants(graph, node)
        // );

        // For every stmt in call block, update lhs and rhs, creating new vars for ssa
        for stmt in self.nodes_in_basic_block(graph, node) {
            // println!("basic_block_loop {}", graph.get_node(stmt));
            self.update_lhs_rhs(graph.get_node_mut(stmt));
        }

        // For every desc call node, rename param to back of var stack
        for s in self.special_descendants(graph, node) {
            let node_data = graph.get_node_mut(s);
            self.update_global_vars_if_nessessary(&node_data.referenced_vars());
            match CallNode::concrete_mut(node_data) {
                Some(CallNode { args }) => {
                    for arg in args {
                        if let Some(stack) = self.stacks.get(arg) {
                            *arg = stack
                                .last()
                                // .unwrap_or(&VarExpr::new(&format!("ERRORRRR_{}", arg)))
                                .unwrap_or_else(|| panic!("{} {:?}", arg, self.stacks))
                                .clone();
                        }
                    }
                }
                _ => {}
            }
        }

        // DFS on dominator tree
        let dominance =
            petgraph::algo::dominators::simple_fast(&graph.graph, graph.get_entry().into());
        for s in graph.dfs(graph.get_entry()) {
            // if node dominates s
            let dominates_s = dominance
                .dominators(petgraph::graph::NodeIndex::new(s.into()))
                .unwrap()
                .collect::<Vec<petgraph::graph::NodeIndex>>();
            if dominates_s.contains(&petgraph::graph::NodeIndex::new(node.into())) {
                match FuncNode::concrete(graph.get_node(s)) {
                    Some(FuncNode { params: _, .. }) => {
                        self.rename(graph, s);
                    }
                    _ => {
                        // If a pred is a branch
                        let preds = graph.preds(s).collect::<Vec<_>>();
                        if preds.len() == 1 {
                            match BranchNode::concrete(graph.get_node(preds[0])) {
                                Some(_) => {
                                    self.rename(graph, s);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // Unwind stack
        for stmt in self.nodes_in_basic_block(graph, node) {
            match FuncNode::concrete(graph.get_node_mut(stmt)) {
                Some(FuncNode { params }) => {
                    for param in params {
                        let stack = self
                            .stacks
                            .get_mut(self.var_mapping.get(param).unwrap_or(param))
                            .unwrap();
                        stack.pop();
                    }
                }
                _ => {}
            }
        }
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
        insert_phi::InsertPhi::default().apply(&mut graph);

        // assert_eq!(
        //     MakeSSA::default().nodes_in_call_block(&graph, 7.into()),
        //     vec![7, 3, 4]
        //         .iter()
        //         .map(|x| (*x).into())
        //         .collect::<Vec<_>>()
        // );

        // assert_eq!(
        //     MakeSSA::default().call_descendants(&graph, 7.into()),
        //     vec![10.into()]
        // );

        MakeSSA::default().apply(&mut graph);

        write_graph(&graph, "make_ssa.dot");
    }

    #[test]
    fn fib() {
        let mut graph = make_fib();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);
        insert_phi::InsertPhi::default().apply(&mut graph);

        // assert_eq!(
        //     MakeSSA::new().nodes_in_call_block(&graph, 5),
        //     vec![5, 1, 2, 3, 4]
        // );

        // assert_eq!(MakeSSA::new().call_descendants(&graph, 5), vec![7]);

        MakeSSA::default().apply(&mut graph);
        write_graph(&graph, "make_ssa.dot");
    }

    #[test]
    fn branch() {
        let mut graph = make_branch();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);
        insert_phi::InsertPhi::default().apply(&mut graph);

        // assert_eq!(
        //     MakeSSA::new().nodes_in_call_block(&graph, 5),
        //     vec![5, 1, 2, 3, 4]
        // );

        // assert_eq!(MakeSSA::new().call_descendants(&graph, 5), vec![7]);

        MakeSSA::default().apply(&mut graph);

        write_graph(&graph, "make_ssa.dot");
    }

    pub fn make_odd_range() -> CFG {
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
    fn odd_range() {
        let mut graph = make_odd_range();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);
        insert_phi::InsertPhi::default().apply(&mut graph);

        assert_eq!(
            MakeSSA::default().nodes_in_basic_block(&mut graph, 7.into()),
            vec![7.into(), 2.into()]
        );
        assert_eq!(
            MakeSSA::default().special_descendants(&mut graph, 7.into()),
            vec![6.into(), 3.into()]
        );
        assert_eq!(
            MakeSSA::default().nodes_in_basic_block(&mut graph, 2.into()),
            vec![2.into()]
        );
        assert_eq!(
            MakeSSA::default().special_descendants(&mut graph, 2.into()),
            vec![6.into(), 3.into()]
        );
        assert_eq!(
            MakeSSA::default().nodes_in_basic_block(&mut graph, 3.into()),
            vec![3.into()]
        );
        assert_eq!(
            MakeSSA::default().nodes_in_basic_block(&mut graph, 8.into()),
            vec![8.into(), 5.into()]
        );
        assert_eq!(
            MakeSSA::default().special_descendants(&mut graph, 8.into()),
            vec![10.into()]
        );
        MakeSSA::transform(&mut graph);
        write_graph(&graph, "make_ssa.dot");
    }

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
    fn even_fib() {
        let mut graph = make_odd_fib();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);
        insert_phi::InsertPhi::default().apply(&mut graph);

        assert_eq!(
            MakeSSA::default().nodes_in_basic_block(&mut graph, 16.into()),
            vec![16.into()]
        );
        assert_eq!(
            MakeSSA::default().special_descendants(&mut graph, 16.into()),
            vec![13.into()]
        );
        // assert_eq!(
        //     MakeSSA::default().nodes_in_basic_block(&mut graph, 11.into()),
        //     vec![11.into()]
        // );
        // assert_eq!(
        //     MakeSSA::default().special_descendants(&mut graph, 11.into()),
        //     vec![8.into()]
        // );
        // assert_eq!(
        //     MakeSSA::default().special_descendants(&mut graph, 2.into()),
        //     vec![6.into(), 3.into()]
        // );
        // assert_eq!(
        //     MakeSSA::default().nodes_in_basic_block(&mut graph, 3.into()),
        //     vec![3.into()]
        // );
        // assert_eq!(
        //     MakeSSA::default().nodes_in_basic_block(&mut graph, 8.into()),
        //     vec![8.into(), 5.into()]
        // );
        // assert_eq!(
        //     MakeSSA::default().special_descendants(&mut graph, 8.into()),
        //     vec![10.into()]
        // );
        MakeSSA::transform(&mut graph);
        // MakeSSA::transform(&mut graph);
        // MakeSSA::transform(&mut graph);
        write_graph(&graph, "make_ssa.dot");
    }

    #[test]
    fn autopopulate_params() {
        let code = r#"
def even_fib(b, d):
    x = a
    y = b
    z = c
    "#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);

        let mut graph = visitor.get_graph();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);
        insert_phi::InsertPhi::default().apply(&mut graph);
        MakeSSA::transform(&mut graph);
        assert_eq!(
            "func(b.0, d.0, a.0, c.0)",
            graph.get_node(graph.entry).to_string()
        );
        // println!("{}", graph.get_node(graph.entry).to_string());
        write_graph(&graph, "make_ssa.dot");
    }
}
