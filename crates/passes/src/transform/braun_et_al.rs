use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;

use crate::*;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

#[derive(Default, Clone)]
pub struct BraunEtAl {
    result: TransformResultType,
    current_def: BTreeMap<VarExpr, BTreeMap<NodeIndex, VarExpr>>,
    var_counter: BTreeMap<VarExpr, usize>,
    added_vars: BTreeSet<VarExpr>,
}

impl BraunEtAl {
    pub(crate) fn write_variable(
        &mut self,
        _graph: &mut CFG,
        variable: &VarExpr,
        block: &NodeIndex,
        value: &VarExpr,
    ) {
        // println!("write variable {} {} {}", variable, block, value);
        // if variable.to_string().contains(".") {
        //     _graph.write_dot("panic.dot");
        //     panic!("{variable:?} {block:?} {value:?}");
        // }
        self.added_vars.insert(value.clone());

        // Create var map if it doesn't exist
        let var_map = match self.current_def.entry(variable.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(BTreeMap::new()),
        };
        var_map.insert(*block, value.clone());
    }

    pub(crate) fn read_variable(
        &mut self,
        graph: &mut CFG,
        variable: &VarExpr,
        block: &NodeIndex,
    ) -> VarExpr {
        // println!(
        //     "read variable {} {} {:?} {:?}",
        //     block,
        //     variable,
        //     self.current_def.get(variable),
        //     self.added_vars,
        // );
        if self.added_vars.contains(variable) {
            return variable.clone();
        }
        if !self.current_def.contains_key(variable) {
            self.current_def.insert(variable.clone(), BTreeMap::new());
        }
        if self.current_def[variable].contains_key(block) {
            // local variable numbering
            self.current_def[variable][block].clone()
        } else {
            // global value numbering
            self.read_variable_recursive(graph, variable, block)
        }
    }

    pub(crate) fn read_variable_recursive(
        &mut self,
        graph: &mut CFG,
        variable: &VarExpr,
        block: &NodeIndex,
    ) -> VarExpr {
        // println!("read variable recursive {} {}", block, variable);
        let mut val;
        if self.added_vars.contains(variable) {
            return variable.clone();
        }

        let preds = graph.preds(*block).collect::<Vec<_>>();
        if preds.len() == 1 {
            val = self.read_variable(graph, variable, &preds[0]);
            /*
             * Note that this branch does not call self.write_variable
             * This is because they may not be the same variable name
             * Take for example, a node that does `a.1 = a.0 + 1`
             * Then the current_defs of this node is `a.1`,
             * which is different than the result of `read_variable_recursive(this_node, a) == a.0`
             */
        } else {
            // Break potential cycles with operandless phi

            // println!("current_def {:?}", self.current_def[variable]);
            val = self.new_phi(graph, block, variable); // add new phi to this block
            self.write_variable(graph, variable, block, &val);
            let confuz = self.add_phi_operands(graph, block, variable, &val);
            // println!("confuz {}", confuz);
            self.write_variable(graph, variable, block, &confuz);
            val = confuz;
            self.write_variable(graph, variable, block, &val)
        }
        val
    }

    pub(crate) fn gen_new_name(&mut self, var: &VarExpr) -> VarExpr {
        let count = *self.var_counter.get(var).unwrap_or(&0);
        self.var_counter.insert(var.clone(), count + 1);
        let name = format!("{}.{}", var.name, count);
        VarExpr::new(&name)
    }

    /// Adds a new phi variable
    pub(crate) fn new_phi(&mut self, graph: &mut CFG, block: &NodeIndex, var: &VarExpr) -> VarExpr {
        let count = *self.var_counter.get(var).unwrap_or(&0);
        self.var_counter.insert(var.clone(), count + 1);

        // println!("new phi {} {}", block, var);
        let name = format!("{}.{}", var.name, count);
        let new_var = VarExpr::new(&name);

        if let Some(FuncNode { params }) = FuncNode::concrete_mut(graph.get_node_mut(*block)) {
            params.push(new_var.clone())
        } else {
            panic!(
                "Block head {} is not a func node, attempted to push {}",
                *block, new_var
            );
        }
        new_var
    }

    /// call(src) -> func(dst)
    /// where var is original variable before renaming
    pub(crate) fn add_phi_operands(
        &mut self,
        graph: &mut CFG,
        block: &NodeIndex,
        var: &VarExpr,
        dst: &VarExpr,
    ) -> VarExpr {
        // println!("add phi operands {} {}", block, var);

        let mut srcs = vec![];

        for pred in graph.preds(*block).collect::<Vec<_>>() {
            let arg = self.read_variable(graph, var, &pred).clone();
            srcs.push(arg.clone());
            if let Some(CallNode { args }) = CallNode::concrete_mut(graph.get_node_mut(pred)) {
                args.push(arg)
            } else {
                panic!(
                    "Func pred {} is not a call node, attempted to push {}",
                    *block, arg
                );
            }
        }
        self.try_remove_trivial_phi(graph, block, dst)
    }

    pub(crate) fn try_remove_trivial_phi(
        &mut self,
        graph: &mut CFG,
        block: &NodeIndex,
        dst: &VarExpr,
    ) -> VarExpr {
        let node = graph.get_node(*block);
        let mut srcs = vec![];
        let mut index = usize::MAX;
        if let Some(FuncNode { params }) = FuncNode::concrete(node) {
            if let Some(idx) = params.iter().position(|v| v == dst) {
                for pred in graph.preds(*block).collect::<Vec<NodeIndex>>() {
                    match CallNode::concrete_mut(graph.get_node_mut(pred)) {
                        Some(CallNode { args }) => {
                            srcs.push(args[idx].clone());
                            index = idx;
                        }
                        _ => panic!(),
                    }
                }
            } else {
                // return dst.clone();
                // println!("not a user of dst {} {}", dst, block)
            }
        } else {
            panic!("not func node")
        }

        let mut same = None;

        // println!("srcs {:?}", srcs);
        for src in srcs {
            // println!("src {}, same {:?}", src, same);
            if let Some(ref s) = same {
                // If unique value
                if *s == src {
                    continue;
                }
            }
            if src == *dst {
                // If self reference
                continue;
            }
            if same.is_some() {
                return dst.clone();
            }
            same = Some(src);
        }
        if same.is_none() {
            return dst.clone();
        }
        let same = same.unwrap();
        // println!("same {}", same);

        let mut users = vec![];
        for idx in graph.nodes() {
            let node = graph.get_node_mut(idx);
            // println!("checking node {}", idx);
            for var in node.referenced_vars_mut() {
                // println!("checking {} for {} to replace with {}", var, dst, same);
                if *var == *dst {
                    *var = same.clone();
                    if !users.contains(&idx) {
                        users.push(idx);
                    }
                }
            }
        }
        // println!("users {:?}", users);

        let node = graph.get_node_mut(*block);
        if let Some(FuncNode { params }) = FuncNode::concrete_mut(node) {
            params.remove(index);
        } else {
            panic!();
        }
        for pred in graph.preds(*block).collect::<Vec<_>>() {
            let node = graph.get_node_mut(pred);
            if let Some(CallNode { args }) = CallNode::concrete_mut(node) {
                args.remove(index);
            } else {
                panic!();
            }
        }

        for user in users {
            let node = graph.get_node(user).clone();
            if let Some(CallNode { args }) = CallNode::concrete(&node) {
                let succs = graph.succs(user).collect::<Vec<_>>();

                if let Some(idx) = args.iter().position(|v| *v == same) {
                    for succ in graph.succs(user).collect::<Vec<NodeIndex>>() {
                        match FuncNode::concrete(graph.get_node(succ)) {
                            Some(FuncNode { params }) => {
                                let new_dst = params[idx].clone();
                                // println!(
                                //     "new_dst {}, same {}, idx {}, args {:?}",
                                //     new_dst, same, idx, args
                                // );
                                self.try_remove_trivial_phi(graph, &succs[0], &new_dst);
                            }
                            _ => panic!(),
                        }
                    }
                }
            }
        }
        same
    }

    pub fn find_external_vars(graph: &mut CFG, successor: NodeIndex) -> Vec<VarExpr> {
        for pred in graph.preds(successor).collect::<Vec<_>>() {
            graph.rmv_edge(pred, successor);
        }
        let dummy_vars = if let Some(FuncNode { params }) =
            FuncNode::concrete(graph.get_node(graph.get_entry()))
        {
            params
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, _)| VarExpr::new(&format!("_dummy_{}", i)))
                .collect::<Vec<_>>()
        } else {
            panic!()
        };
        // println!("dummy vars {:?}", dummy_vars);
        {
            let new_call = graph.add_node(CallNode {
                args: dummy_vars.clone(),
            });
            let new_func = graph.add_node(FuncNode {
                params: dummy_vars.clone(),
            });
            graph.add_edge(new_func, new_call, NoneEdge.into());
            graph.add_edge(new_call, successor, NoneEdge.into());
            graph.set_entry(new_func);
            /// Clears all args and params from all call and func nodes that have a predecessor
            pub(crate) fn clear_all_phis(graph: &mut CFG) {
                for node in graph.nodes() {
                    if graph.preds(node).count() == 0 {
                        continue;
                    }
                    let node_data = graph.get_node_mut(node);
                    match FuncNode::concrete_mut(node_data) {
                        Some(FuncNode { params }) => {
                            params.clear();
                        }
                        None => {}
                    }
                    match CallNode::concrete_mut(node_data) {
                        Some(CallNode { args }) => {
                            args.clear();
                        }
                        None => {}
                    }
                }
            }
            BraunEtAl::transform(graph);

            graph.write_dot(&format!("braun_{}_.dot", successor));
        }
        // Filter for external vars
        if let Some(FuncNode { params }) = FuncNode::concrete(graph.get_node(graph.get_entry())) {
            params
                .iter()
                .filter(|x| !dummy_vars.contains(x))
                .cloned()
                .collect()
        } else {
            panic!()
        }
    }
}

impl Transform for BraunEtAl {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        // TODO: change API to store a reference
        let node_indexes = graph.nodes().collect::<Vec<_>>();
        for idx in &node_indexes {
            let mut node = graph.get_node(*idx).clone();
            for var in node.declared_vars_mut() {
                let new_var = self.gen_new_name(var);
                self.write_variable(graph, var, idx, &new_var);
                // println!("above -> {:?}", self.current_def);
            }
        }

        for idx in &node_indexes {
            {
                // Rename all variable definitions/writes
                let node = graph.get_node(*idx).clone();
                let vars = node.declared_vars();
                let mut new_vars = VecDeque::new();
                for var in vars {
                    new_vars.push_back(self.read_variable(graph, var, idx));
                    // println!(
                    //     "middle -> {} var {} -> {:?} {:?}",
                    //     idx,
                    //     var,
                    //     new_vars.back(),
                    //     self.current_def
                    // );
                }
                let node = graph.get_node_mut(*idx);
                for var in node.declared_vars_mut() {
                    *var = new_vars
                        .pop_front()
                        .unwrap_or(VarExpr::new(&format!("ERRRRROR_{}", var)));
                    // println!(
                    //     "middle1 -> {} var {} -> {:?} {:?}",
                    //     idx,
                    //     var,
                    //     new_vars.back(),
                    //     self.current_def
                    // );
                }
            }
            {
                // Rename all variable references/reads
                let node = graph.get_node(*idx).clone();
                let mut new_vars = VecDeque::new();
                let vars = node.referenced_vars();
                // println!("references -> {} {} {:?}", idx, node, vars);
                for var in vars {
                    // Recursive variable read, as assign nodes may be dependent on itself
                    // E.g. i = i + 1 -> i1 = i + 1 -> need to avoid i1 = i1 + 1
                    new_vars.push_back(self.read_variable_recursive(graph, var, idx));
                    // println!(
                    //     "{} var {} -> {:?} {:?}",
                    //     idx,
                    //     var,
                    //     new_vars.back(),
                    //     self.current_def
                    // );
                }
                // println!("{} new_vars {:?}", idx, new_vars);

                // Update every variable with its value at this node
                // E.g. the last SSA to this variable name from predecessor node
                let node = graph.get_node_mut(*idx);
                for var in node.referenced_vars_mut() {
                    // print!("other order {}, ", var);
                    *var = new_vars.pop_front().unwrap();
                }
                // println!();
            }
        }

        // Restore initial function's args to their pre-rename names
        let mut og_mapping: BTreeMap<VarExpr, VarExpr> = BTreeMap::new();
        if let Some(FuncNode { params }) =
            FuncNode::concrete_mut(graph.get_node_mut(graph.get_entry()))
        {
            // println!("restoring original names {:?}", params);
            for param in params {
                for (var, block_def) in &self.current_def {
                    for (_block, new_var) in block_def {
                        if new_var == param {
                            og_mapping.insert(new_var.clone(), var.clone());
                        }
                    }
                }
            }
        } else {
            panic!();
        };
        // println!("og_mapping {:?}", og_mapping);
        // println!("{:#?}", self.current_def);
        for idx in &node_indexes {
            let node = graph.get_node_mut(*idx);
            for var in node.referenced_vars_mut() {
                if og_mapping.contains_key(var) {
                    *var = og_mapping[var].clone();
                }
            }
            for var in node.declared_vars_mut() {
                if og_mapping.contains_key(var) {
                    *var = og_mapping[var].clone();
                }
            }
        }
        // println!("current_def {:#?}", self.current_def);
        &self.result
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::tests::*;
    use crate::transform::*;

    #[test]
    fn range() {
        let mut graph = make_range();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);

        let mut pass = BraunEtAl::default();
        // assert_eq!(pass.get_block_head(&mut graph, 4.into()), 6.into());

        // pass.write_variable(
        //     &mut graph,
        //     &VarExpr::new("i"),
        //     &1.into(),
        //     &VarExpr::new("i0"),
        // );
        // let result = pass.read_variable(&mut graph, &VarExpr::new("i"), &2.into());
        // println!("result {}", result);

        pass.apply(&mut graph);

        write_graph(&graph, "braun.dot");
    }

    #[test]
    fn branch() {
        let mut graph = make_branch();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);

        let mut pass = BraunEtAl::default();

        pass.apply(&mut graph);

        let result = pass.read_variable(&mut graph, &VarExpr::new("b"), &4.into());
        // println!("result {}", result);
        let result = pass.read_variable(&mut graph, &VarExpr::new("b"), &4.into());
        // println!("result {}", result);
        let result = pass.read_variable(&mut graph, &VarExpr::new("b"), &5.into());
        // println!("result {}", result);
        let result = pass.read_variable(&mut graph, &VarExpr::new("b"), &6.into());
        // println!("result {}", result);
        let result = pass.read_variable(&mut graph, &VarExpr::new("b"), &6.into());
        // println!("result {}", result);

        // println!("current_def {:#?}", pass.current_def);

        write_graph(&graph, "braun.dot");
    }

    #[test]
    fn fib() {
        let mut graph = make_even_fib();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);

        let mut pass = BraunEtAl::default();

        pass.apply(&mut graph);

        // let result = pass.read_variable(&mut graph, &VarExpr::new("n"), &4.into());
        // println!("result {}", result);

        // let result = pass.read_variable(&mut graph, &VarExpr::new("i"), &4.into());
        // println!("result {}", result);

        // let result = pass.read_variable(&mut graph, &VarExpr::new("i"), &10.into());
        // println!("result {}", result);

        // pass.try_remove_trivial_phi(&mut graph, &13.into(), &VarExpr::new("n.1"));
        // pass.try_remove_trivial_phi(&mut graph, &13.into(), &VarExpr::new("a.3"));
        // pass.try_remove_trivial_phi(&mut graph, &13.into(), &VarExpr::new("b.2"));

        // println!("current_def {:#?}", pass.current_def);

        assert!(graph.to_dot().contains("call(i.1, a.1, b.1)"));
        write_graph(&graph, "braun.dot");
    }

    #[test]
    fn linear() {
        let mut graph = make_linear();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);

        let mut pass = BraunEtAl::default();

        pass.apply(&mut graph);
        println!(
            "final {}",
            pass.read_variable(&mut graph, &VarExpr::new("a"), &5.into())
        );
        println!(
            "final {}",
            pass.read_variable(&mut graph, &VarExpr::new("d"), &5.into())
        );

        // println!("read_vars {:?}", pass.read_vars);
        // println!("wrote_vars {:?}", pass.wrote_vars);
        // println!("current_def {:#?}", pass.current_def);

        assert!(graph.to_dot().contains("a.1 = (c.0 + 23)"));
        assert!(graph.to_dot().contains("c.1 = (a.1 + d)"));
        write_graph(&graph, "braun.dot");
    }

    #[test]
    fn aug_assign() {
        let mut graph = make_aug_assign();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);

        let mut pass = BraunEtAl::default();

        pass.apply(&mut graph);
        // println!(
        //     "final {}",
        //     pass.read_variable(&mut graph, &VarExpr::new("a"), &5.into())
        // );
        // println!(
        //     "final {}",
        //     pass.read_variable(&mut graph, &VarExpr::new("d"), &5.into())
        // );

        // println!("read_vars {:?}", pass.read_vars);
        // println!("wrote_vars {:?}", pass.wrote_vars);
        // println!("current_def {:#?}", pass.current_def);

        assert!(graph.to_dot().contains("return (a.1)"));
        write_graph(&graph, "braun.dot");
    }
}
