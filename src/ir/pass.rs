pub mod insert_func;

use super::graph::DiGraph;

pub trait Transform {
    fn transform(&self, graph: &mut DiGraph);
}
