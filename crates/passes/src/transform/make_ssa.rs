use std::collections::{BTreeMap, BTreeSet};

use crate::*;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

#[derive(Debug)]
pub struct MakeSSA {
    visited: BTreeSet<NodeIndex>,
    var_counter: BTreeMap<VarExpr, usize>,
    stacks: BTreeMap<VarExpr, Vec<VarExpr>>,
    var_mapping: BTreeMap<VarExpr, VarExpr>,
    pub(crate) global_vars: Vec<VarExpr>,

    separater: &'static str,
    result: TransformResultType,
}

impl Default for MakeSSA {
    fn default() -> Self {
        Self {
            visited: BTreeSet::new(),
            var_counter: BTreeMap::new(),
            stacks: BTreeMap::new(),
            var_mapping: BTreeMap::new(),
            global_vars: vec![],
            separater: ".",
            result: TransformResultType::default(),
        }
    }
}

impl Transform for MakeSSA {
    /// Applies transformation
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        self.rename(graph, graph.get_entry());

        // If a global var is not in initial func call, add it
        let node = graph.get_node_mut(graph.get_entry());
        println!("makessa node {}", node);
        println!("makessa global vars {:?}", self.global_vars);
        match node {
            Node::Func(FuncNode { params }) => {
                for var in &self.global_vars {
                    if !params.contains(var) {
                        println!("makessa pushing {}", var);
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
        println!("test_rename starting at {}", node);

        self.rename(graph, node);

        // If a global var is not in initial func call, add it
        let node = graph.get_node_mut(graph.get_entry());
        println!("makessa node {}", node);
        println!("makessa global vars {:?}", self.global_vars);
        match node {
            Node::Func(FuncNode { params }) => {
                for var in &self.global_vars {
                    if !params.contains(var) {
                        // println!("makessa pushing {}", var);
                        params.push(var.clone());
                    }
                }
            }
            _ => panic!(),
        }

        // println!("self stats {:#?}", self);

        // Map global vars to their names before ssa
        self.global_vars
            .iter()
            .map(|x| self.var_mapping.get(x).unwrap().clone())
            .collect()
    }

    /// Make revert mapping
    fn make_revert_mapping(&self, expr: &Expr) -> BTreeMap<VarExpr, Expr> {
        let mut ret = BTreeMap::new();
        for var in expr.get_vars() {
            ret.insert(
                var.clone(),
                Expr::Var(VarExpr::new(
                    var.name.split(self.separater).collect::<Vec<_>>()[0],
                )),
            );
        }
        ret
    }

    /// Revert SSA by removing separator from variable names
    /// Only retains correctness if reverted immediately after transforming to SSA
    pub fn revert_ssa_dangerous(&self, graph: &mut CFG) {
        for node in graph.dfs(graph.get_entry()) {
            match graph.get_node_mut(node) {
                Node::Assign(AssignNode { lvalue, rvalue }) => {
                    *lvalue =
                        VarExpr::new(lvalue.name.split(self.separater).collect::<Vec<_>>()[0]);
                    rvalue.backwards_replace(&self.make_revert_mapping(rvalue));
                }
                Node::Func(FuncNode { params }) => {
                    for param in params {
                        *param =
                            VarExpr::new(param.name.split(self.separater).collect::<Vec<_>>()[0]);
                    }
                }
                Node::Call(CallNode { args, .. }) => {
                    for arg in args {
                        *arg = VarExpr::new(arg.name.split(self.separater).collect::<Vec<_>>()[0]);
                    }
                }
                Node::Return(TermNode { values }) | Node::Yield(TermNode { values }) => {
                    for value in values {
                        value.backwards_replace(&self.make_revert_mapping(value));
                    }
                }
                Node::Branch(BranchNode { cond }) => {
                    cond.backwards_replace(&self.make_revert_mapping(cond));
                }
            }
        }
    }

    /// Get nodes within call block
    pub(crate) fn nodes_in_call_block(&self, graph: &CFG, node: NodeIndex) -> Vec<NodeIndex> {
        graph.descendants_internal(node, &|n| match n {
            Node::Call(_) => false,
            _ => true,
        })
    }

    /// Gets descendant call nodes
    pub(crate) fn call_descendants(&self, graph: &CFG, node: NodeIndex) -> Vec<NodeIndex> {
        graph.descendants_leaves(node, &|n| match n {
            Node::Call(_) => true,
            _ => false,
        })
    }

    // Make basic block subtree
    pub(crate) fn nodes_in_basic_block(&self, graph: &CFG, source: NodeIndex) -> Vec<NodeIndex> {
        // let mut stack = match graph.get_node(source) {
        //     Node::Func(_) => graph.succ(source).collect::<Vec<NodeIndex>>(),
        //     _ => {
        //         vec![source]
        //     }
        // };
        let mut stack = vec![source];

        let mut result = vec![];

        while let Some(node) = stack.pop() {
            let node_data = graph.get_node(node);
            match node_data {
                Node::Call(_) => {}
                Node::Branch(_) => {
                    result.push(node);
                }
                _ => {
                    result.push(node);

                    for succ in graph.succ(node) {
                        stack.push(succ);
                    }
                }
            }
        }
        result
    }

    pub(crate) fn special_descendants(&self, graph: &CFG, source: NodeIndex) -> Vec<NodeIndex> {
        // let mut stack = graph.succ(source).collect::<Vec<NodeIndex>>();
        let mut stack = match graph.get_node(source) {
            Node::Call(_) => graph.succ(source).collect::<Vec<NodeIndex>>(),
            _ => {
                vec![source]
            }
        };
        let mut result = vec![];

        while let Some(node) = stack.pop() {
            let node_data = graph.get_node(node);
            match node_data {
                Node::Call(_) => {
                    result.push(node);
                }
                Node::Branch(_) => {
                    for succ in graph.succ(node) {
                        result.push(succ)
                    }
                }
                _ => {
                    for succ in graph.succ(node) {
                        stack.push(succ);
                    }
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
        // println!("self stacks {:?}", self.stacks);

        // Update var mapping
        self.var_mapping.insert(new_var.clone(), var.clone());

        // println!("gen_name after {:?}", self.stacks);
        new_var
    }

    /// Assert read vars are apart of stacks, otherwise it is a global var
    fn update_global_vars_if_nessessary(&mut self, vars: &Vec<VarExpr>) {
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
    fn update_lhs_rhs(&mut self, stmt: &mut Node) {
        match stmt {
            Node::Assign(AssignNode { lvalue, rvalue }) => {
                self.update_global_vars_if_nessessary(&rvalue.get_vars());
                // Note that old mapping is used for rvalue
                rvalue.backwards_replace(&self.make_mapping());
                *lvalue = self.gen_name(lvalue);
            }
            Node::Branch(BranchNode { cond }) => {
                self.update_global_vars_if_nessessary(&cond.get_vars());
                cond.backwards_replace(&self.make_mapping());
            }
            Node::Yield(TermNode { values }) | Node::Return(TermNode { values }) => {
                for value in values {
                    self.update_global_vars_if_nessessary(&value.get_vars());
                    value.backwards_replace(&self.make_mapping());
                }
            }
            // Unused as this func is called within call block
            Node::Call(CallNode { args }) => {
                self.update_global_vars_if_nessessary(args);
            }
            Node::Func(FuncNode { params }) => {
                for param in params {
                    *param = self.gen_name(param);
                }
            }
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

        println!("rename node {}", node);

        // For every stmt in call block, update lhs and rhs, creating new vars for ssa
        for stmt in self.nodes_in_basic_block(graph, node) {
            self.update_lhs_rhs(graph.get_node_mut(stmt));
        }

        // For every desc call node, rename param to back of var stack
        for s in self.special_descendants(graph, node) {
            match graph.get_node_mut(s) {
                Node::Call(CallNode { args }) => {
                    println!("call_descendants {}", s);
                    for arg in args {
                        if let Some(stack) = self.stacks.get(arg) {
                            *arg = stack
                                .last()
                                .unwrap_or(&VarExpr::new(&format!("ERRORRRR_{}", arg)))
                                // .unwrap_or_else(|| panic!("{} {:?}", arg, self.stacks))
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
                match graph.get_node(s) {
                    Node::Func(FuncNode { params: _, .. }) => {
                        self.rename(graph, s);
                    }
                    _ => {
                        // If a pred is a branch
                        let preds = graph.pred(s).collect::<Vec<_>>();
                        if preds.len() == 1 {
                            match graph.get_node(preds[0]) {
                                Node::Branch(_) => {
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
        match graph.get_node(node) {
            Node::Func(FuncNode { params: args }) => {
                for arg in args {
                    let stack = self
                        .stacks
                        .get_mut(
                            self.var_mapping
                                .get(arg)
                                .expect(&format!("{} {:?}", arg, self.var_mapping)),
                        )
                        .unwrap();
                    stack.pop();
                }
            }
            _ => {}
        }
        for stmt in self.nodes_in_call_block(graph, node) {
            match graph.get_node_mut(stmt) {
                Node::Assign(AssignNode { lvalue, .. }) => {
                    let stack = self
                        .stacks
                        .get_mut(self.var_mapping.get(lvalue).unwrap())
                        .unwrap();
                    stack.pop();
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

        assert_eq!(
            MakeSSA::default().nodes_in_call_block(&graph, 7.into()),
            vec![7, 3, 4]
                .iter()
                .map(|x| (*x).into())
                .collect::<Vec<_>>()
        );

        assert_eq!(
            MakeSSA::default().call_descendants(&graph, 7.into()),
            vec![10.into()]
        );

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
        MakeSSA::default().revert_ssa_dangerous(&mut graph);
        MakeSSA::default().apply(&mut graph);
        MakeSSA::default().apply(&mut graph);
        MakeSSA::default().revert_ssa_dangerous(&mut graph);
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
        let mut graph = make_odd_range();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);
        insert_phi::InsertPhi::default().apply(&mut graph);

        // assert_eq!(
        //     MakeSSA::default().nodes_in_basic_block(&mut graph, 7.into()),
        //     vec![7.into(), 2.into()]
        // );
        // assert_eq!(
        //     MakeSSA::default().special_descendants(&mut graph, 7.into()),
        //     vec![6.into(), 3.into()]
        // );
        // assert_eq!(
        //     MakeSSA::default().nodes_in_basic_block(&mut graph, 2.into()),
        //     vec![2.into()]
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
        MakeSSA::transform(&mut graph);
        MakeSSA::transform(&mut graph);
        write_graph(&graph, "make_ssa.dot");
    }
}
