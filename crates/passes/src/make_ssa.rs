use super::*;
use crate::ir::expr::*;
use crate::ir::graph::*;

pub struct MakeSSA {}

impl MakeSSA {
    /// Gets block of statements
    pub(crate) fn block(&self, graph: &DiGraph, node: usize) -> Vec<usize> {
        return graph.dfs(node, &|n| match n {
            Node::Call(_) => false,
            _ => true,
        });
    }
}

impl Transform for MakeSSA {
    fn transform(&self, graph: &mut DiGraph) {
        for node in graph.nodes() {
            match graph.get_node_mut(node) {
                Node::Assign(assign) => {
                    assign.lvalue = VarExpr::new("t");
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::tests::*;

    #[test]
    fn main() {
        let mut graph = make_range();

        insert_func::InsertFuncNodes {}.transform(&mut graph);
        MakeSSA {}.transform(&mut graph);

        assert_eq!(MakeSSA {}.block(&graph, 5), vec![5, 1, 2, 3, 4]);

        let result = MakeSSA {}.transform(&mut graph);

        println!("result {:?}", result);

        write_graph(&graph, "make_ssa.dot");
    }
}
