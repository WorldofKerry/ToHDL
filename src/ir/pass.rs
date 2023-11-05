pub mod insert_func;
pub mod insert_call;
pub mod make_ssa;
pub mod insert_phi;

use super::graph::DiGraph;

pub trait Transform {
    fn transform(&self, graph: &mut DiGraph);
}
