use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::*;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

#[derive(Default)]
pub struct BraunEtAl {
    result: TransformResultType,
    current_def: BTreeMap<VarExpr, BTreeMap<NodeIndex, VarExpr>>,
    pub(crate) graph: CFG,
}

impl BraunEtAl {
    pub(crate) fn write_variable(
        &mut self,
        variable: &VarExpr,
        block: &NodeIndex,
        value: &VarExpr,
    ) {
        println!("write variable {} {} {}", variable, block, value);
        let map = match self.current_def.entry(variable.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(BTreeMap::new()),
        };
        map.insert(*block, value.clone());
    }

    pub(crate) fn read_variable(&mut self, variable: &VarExpr, block: &NodeIndex) -> VarExpr {
        println!("read variable {} {}", block, variable);
        if !self.current_def.contains_key(variable) {
            self.current_def.insert(variable.clone(), BTreeMap::new());
        }
        if self.current_def[variable].contains_key(block) {
            // local variable numbering
            self.current_def[variable][block].clone()
        } else {
            // global value numbering
            self.read_variable_recursive(variable, block)
        }
    }

    pub(crate) fn read_variable_recursive(
        &mut self,
        variable: &VarExpr,
        block: &NodeIndex,
    ) -> VarExpr {
        println!("read variable recursive {} {}", block, variable);
        // assume complete CFG
        let mut val;
        // break potential cycles with operandless phi
        val = self.new_phi(block, variable); // add new phi to this block
        self.write_variable(variable, block, &val);
        self.add_phi_operands(block, variable);
        self.write_variable(variable, block, &val);
        val
    }

    /// Given a node, get its block head
    pub(crate) fn get_block_head(&self, node: NodeIndex) -> NodeIndex {
        let mut cur = node;
        while !FuncNode::downcastable(self.graph.get_node(cur)) {
            let preds = self.graph.pred(cur).collect::<Vec<_>>();
            assert!(preds.len() == 1);
            cur = preds[0];
        }
        cur.clone()
    }

    /// Adds a new phi variable
    pub(crate) fn new_phi(&mut self, block: &NodeIndex, var: &VarExpr) -> VarExpr {
        println!("new phi {} {}", block, var);
        let name = format!("{}.{}", var.name, 778);
        let new_var = VarExpr::new(&name);

        let block_head_idx = self.get_block_head(block.clone());
        if let Some(FuncNode { params }) =
            FuncNode::concrete_mut(self.graph.get_node_mut(block_head_idx))
        {
            params.push(new_var.clone())
        } else {
            panic!("Block head is not a func node")
        }
        new_var
    }

    pub(crate) fn add_phi_operands(&mut self, block: &NodeIndex, var: &VarExpr) {
        println!("add phi operands {} {}", block, var);
        let block_head_idx = self.get_block_head(block.clone());
        for pred in self.graph.pred(block_head_idx).collect::<Vec<_>>() {
            let arg = self.read_variable(var, &self.get_block_head(pred)).clone();
            if let Some(CallNode { args }) = CallNode::concrete_mut(self.graph.get_node_mut(pred)) {
                args.push(arg)
            } else {
                panic!("Func pred is not a call node")
            }
        }
    }
}

impl Transform for BraunEtAl {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        // TODO: change API to store a reference
        self.graph = graph.clone();
        *graph = self.graph.clone();
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
        pass.graph = graph;

        assert_eq!(pass.get_block_head(4.into()), 6.into());

        let result = pass.read_variable(&VarExpr::new("i"), &2.into());

        println!("result {}", result);

        write_graph(&pass.graph, "braun.dot");
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
