use super::*;

pub struct InsertFuncNodes {}

impl Transform for InsertFuncNodes {
    fn transform(&self, graph: &mut DiGraph) {
        for succ in graph.succ(0) {
            println!("succ: {}", succ.1);
        }
    }
}
