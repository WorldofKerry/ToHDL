use tohdl_ir::graph::DiGraph;

pub trait Transform {
    fn transform(&self, graph: &mut DiGraph);
}
