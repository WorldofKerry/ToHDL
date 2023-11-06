pub mod insert_call;
pub mod insert_func;
pub mod insert_phi;
pub mod make_ssa;

use super::graph::DiGraph;

pub trait Transform {
    fn transform(&self, graph: &mut DiGraph);
}
