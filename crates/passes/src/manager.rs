use crate::*;

pub struct PassManager {
    passes: Vec<fn(&mut DiGraph) -> TransformResultType>,
    result: TransformResultType,
}

impl Default for PassManager {
    fn default() -> Self {
        Self {
            passes: vec![],
            result: TransformResultType::default(),
        }
    }
}

impl PassManager {
    // Takes a transform constructor and adds it to the manager
    pub fn add_pass(&mut self, pass: fn(&mut DiGraph) -> TransformResultType) {
        self.passes.push(pass);
    }
}

impl Transform for PassManager {
    fn apply(&mut self, graph: &mut DiGraph) -> &TransformResultType {
        let limit = 10;
        let mut did_work = false;
        for pass in &self.passes {
            let result = pass(graph);
            did_work |= result.did_work;
        }
        &self.result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{optimize::*, tests::*, transform::*};

    #[test]
    fn main() {
        let mut manager = PassManager::default();

        manager.add_pass(InsertFuncNodes::transform);
        manager.add_pass(InsertCallNodes::transform);
        manager.add_pass(InsertPhi::transform);
        manager.add_pass(MakeSSA::transform);
        manager.add_pass(RemoveRedundantCalls::transform);

        let mut graph = make_range();
        manager.apply(&mut graph);

        write_graph(&graph, "manager.dot")
    }

    #[test]
    fn odd_fib() {
        let mut manager = PassManager::default();

        manager.add_pass(InsertFuncNodes::transform);
        manager.add_pass(InsertCallNodes::transform);
        manager.add_pass(InsertPhi::transform);
        manager.add_pass(MakeSSA::transform);
        manager.add_pass(RemoveRedundantCalls::transform);

        let mut graph = make_odd_fib();
        manager.apply(&mut graph);

        write_graph(&graph, "manager.dot")
    }
}
