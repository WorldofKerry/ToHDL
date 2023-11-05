pub mod insert_func;
pub mod insert_call;

use super::graph::DiGraph;

pub trait Transform {
    fn transform(&self, graph: &mut DiGraph);
}
