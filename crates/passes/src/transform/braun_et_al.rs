use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::*;
use tohdl_ir::expr::*;
use tohdl_ir::graph::*;

#[derive(Default)]
pub struct BraunEtAl {
    result: TransformResultType,
    current_def: BTreeMap<VarExpr, BTreeMap<NodeIndex, Expr>>,
    pub(crate) graph: CFG,
}

impl BraunEtAl {
    pub(crate) fn write_variable(&mut self, variable: &VarExpr, block: &NodeIndex, value: &Expr) {
        // self.current_def[variable][block] = value.clone();
        let map = match self.current_def.entry(variable.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(BTreeMap::new()),
        };
        map.insert(*block, value.clone());
    }

    pub(crate) fn read_variable(&mut self, variable: &VarExpr, block: &NodeIndex) -> &Expr {
        if self.current_def[variable].contains_key(block) {
            // local variable numbering
            &self.current_def[variable][block]
        } else {
            // global value numbering
            self.read_variable_recursive(variable, block)
        }
    }

    pub(crate) fn read_variable_recursive(
        &mut self,
        variable: &VarExpr,
        block: &NodeIndex,
    ) -> &Expr {
        // assume complete CFG
        let mut val;
        // break potential cycles with operandless phi
        val = Expr::Var(VarExpr::new("new_var")); // add new phi to this block
        self.write_variable(variable, block, &val);
        val = Expr::Var(VarExpr::new("result_of_phi")); // update preds
        self.write_variable(variable, block, &val);
        todo!()
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
