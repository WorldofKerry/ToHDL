use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::*;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

#[derive(Default, Clone)]
pub struct BraunEtAl {
    result: TransformResultType,
    current_def: BTreeMap<VarExpr, BTreeMap<NodeIndex, VarExpr>>,
    var_counter: BTreeMap<VarExpr, usize>,
    read_vars: BTreeMap<NodeIndex, Vec<VarExpr>>,
    wrote_vars: BTreeMap<NodeIndex, Vec<VarExpr>>,
}

impl BraunEtAl {
    pub(crate) fn write_variable(
        &mut self,
        graph: &mut CFG,
        variable: &VarExpr,
        block: &NodeIndex,
        value: &VarExpr,
    ) {
        println!("write variable {} {} {}", variable, block, value);

        let block_head_idx = self.get_block_head(graph, *block);

        // Create var map if it doesn't exist
        let var_map = match self.current_def.entry(variable.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(BTreeMap::new()),
        };
        var_map.insert(block_head_idx, value.clone());
    }

    pub(crate) fn read_variable(
        &mut self,
        graph: &mut CFG,
        variable: &VarExpr,
        block: &NodeIndex,
    ) -> VarExpr {
        println!("read variable {} {}", block, variable);
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
        println!("read variable recursive {} {}", block, variable);
        // assume complete CFG
        let val;
        let preds = graph.pred(*block).collect::<Vec<_>>();
        if preds.len() == 1 {
            val = self.read_variable(graph, variable, &preds[0]);
        } else {
            // break potential cycles with operandless phi
            val = self.new_phi(graph, block, variable); // add new phi to this block
            self.write_variable(graph, variable, block, &val);
            self.add_phi_operands(graph, block, variable);
            self.write_variable(graph, variable, block, &val);
        }
        val
    }

    /// Given a node, get its block head
    pub(crate) fn get_block_head(&self, graph: &mut CFG, node: NodeIndex) -> NodeIndex {
        let mut cur = node;
        loop {
            let preds = graph.pred(cur).collect::<Vec<_>>();
            // If node has zero preds or multiple preds (e.g. func node)
            if preds.len() == 0 || preds.len() > 1 {
                return cur;
            }
            // If node's pred  has multiple succs (e.g. branch node)
            if graph.succ(preds[0]).collect::<Vec<_>>().len() > 1 {
                return cur;
            }
            cur = preds[0];
        }
    }

    /// Adds a new phi variable
    pub(crate) fn new_phi(&mut self, graph: &mut CFG, block: &NodeIndex, var: &VarExpr) -> VarExpr {
        let count = *self.var_counter.get(var).unwrap_or(&0);
        self.var_counter.insert(var.clone(), count + 1);

        println!("new phi {} {}", block, var);
        let name = format!("{}.{}", var.name, count);
        let new_var = VarExpr::new(&name);

        let block_head_idx = self.get_block_head(graph, block.clone());
        if let Some(FuncNode { params }) =
            FuncNode::concrete_mut(graph.get_node_mut(block_head_idx))
        {
            params.push(new_var.clone())
        } else {
            println!(
                "Block head {} is not a func node, attempted to push {}",
                block_head_idx, new_var
            );
        }
        new_var
    }

    pub(crate) fn add_phi_operands(&mut self, graph: &mut CFG, block: &NodeIndex, var: &VarExpr) {
        println!("add phi operands {} {}", block, var);
        let block_head_idx = self.get_block_head(graph, block.clone());
        for pred in graph.pred(block_head_idx).collect::<Vec<_>>() {
            let pred_head_idx = &self.get_block_head(graph, pred);
            let arg = self.read_variable(graph, var, pred_head_idx).clone();
            if let Some(CallNode { args }) = CallNode::concrete_mut(graph.get_node_mut(pred)) {
                args.push(arg)
            } else {
                println!(
                    "Func pred {} is not a call node, attempted to push {}",
                    block_head_idx, arg
                );
            }
        }
    }
}

impl Transform for BraunEtAl {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        // TODO: change API to store a reference
        let node_indexes = graph.nodes().collect::<Vec<_>>();
        for idx in &node_indexes {
            let mut read_vars = match self.read_vars.entry(*idx) {
                Entry::Occupied(o) => o.into_mut(),
                Entry::Vacant(v) => v.insert(vec![]),
            }
            .clone();
            let mut wrote_vars = match self.wrote_vars.entry(*idx) {
                Entry::Occupied(o) => o.into_mut(),
                Entry::Vacant(v) => v.insert(vec![]),
            }
            .clone();
            let mut node = graph.get_node(*idx).clone();
            for var in node.referenced_vars() {
                let new_var = self.read_variable(graph, var, idx);
                read_vars.push(new_var);
            }
            for var in node.defined_vars_mut() {
                self.write_variable(graph, var, idx, var);
            }
            self.read_vars.insert(*idx, read_vars);
            self.wrote_vars.insert(*idx, wrote_vars);
        }

        println!("read_vars {:?}", self.read_vars);
        println!("wrote_vars {:?}", self.wrote_vars);
        println!("current_def {:#?}", self.current_def);
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
        assert_eq!(pass.get_block_head(&mut graph, 4.into()), 6.into());

        pass.write_variable(
            &mut graph,
            &VarExpr::new("i"),
            &1.into(),
            &VarExpr::new("i0"),
        );
        let result = pass.read_variable(&mut graph, &VarExpr::new("i"), &2.into());

        println!("result {}", result);

        write_graph(&graph, "braun.dot");
    }

    #[test]
    fn branch_manual() {
        let mut graph = make_branch();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);

        let mut pass = BraunEtAl::default();

        assert_eq!(pass.get_block_head(&mut graph, 1.into()), 0.into());
        assert_eq!(pass.get_block_head(&mut graph, 6.into()), 2.into());
        assert_eq!(pass.get_block_head(&mut graph, 4.into()), 5.into());

        let result = pass.read_variable(&mut graph, &VarExpr::new("a"), &0.into());
        println!("result {}", result);
        let result = pass.read_variable(&mut graph, &VarExpr::new("a"), &1.into());
        println!("result {}", result);
        pass.write_variable(
            &mut graph,
            &VarExpr::new("b"),
            &3.into(),
            &VarExpr::new("a"),
        );
        pass.write_variable(
            &mut graph,
            &VarExpr::new("b"),
            &2.into(),
            &VarExpr::new("1"),
        );
        let result = pass.read_variable(&mut graph, &VarExpr::new("b"), &4.into());
        println!("result {}", result);

        write_graph(&graph, "braun.dot");
    }

    #[test]
    fn branch() {
        let mut graph = make_branch();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);

        let mut pass = BraunEtAl::default();

        assert_eq!(pass.get_block_head(&mut graph, 1.into()), 0.into());
        assert_eq!(pass.get_block_head(&mut graph, 6.into()), 2.into());
        assert_eq!(pass.get_block_head(&mut graph, 4.into()), 5.into());

        pass.apply(&mut graph);

        let result = pass.read_variable(&mut graph, &VarExpr::new("b"), &4.into());
        println!("result {}", result);

        let result = pass.read_variable(&mut graph, &VarExpr::new("a"), &1.into());
        println!("result {}", result);

        write_graph(&graph, "braun.dot");
    }

    #[test]
    fn fib() {
        let mut graph = make_fib();

        insert_func::InsertFuncNodes::default().apply(&mut graph);
        insert_call::InsertCallNodes::default().apply(&mut graph);

        // assert_eq!(BraunEtAl {}.dominance_frontier(&graph, 2), vec![5]);

        // assert_eq!(BraunEtAl {}.get_variables(&graph), vec![VarExpr::new("i")]);

        // let result = BraunEtAldefault().apply_to_var(VarExpr::new("i"), 0, &mut graph);

        // println!("result {:?}", result);

        BraunEtAl::default().apply(&mut graph);

        write_graph(&graph, "braun.dot");
    }
}
