use crate::*;

#[derive(Default)]
pub struct PassManager {
    passes: Vec<fn(&mut CFG) -> TransformResultType>,
    result: TransformResultType,
    log: bool,
}

impl PassManager {
    /// Takes a transform constructor and adds it to the manager
    pub fn add_pass(&mut self, pass: fn(&mut CFG) -> TransformResultType) {
        self.passes.push(pass);
    }

    /// Create a debug manager
    pub fn log() -> Self {
        Self {
            passes: vec![],
            result: Default::default(),
            log: true,
        }
    }
}

impl PassManager {
    fn log_pass(&self, result: &TransformResultType) {
        println!("{}", result);
    }
}

impl BasicTransform for PassManager {
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        for pass in &self.passes {
            let result = pass(graph);
            self.result.elapsed_time += result.elapsed_time;
            self.result.did_work |= result.did_work;

            if self.log {
                self.log_pass(&result);
            }
        }
        &self.result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tests::*, transform::*};

    #[test]
    fn main() {
        let mut manager = PassManager::default();

        manager.add_pass(InsertFuncNodes::transform);
        manager.add_pass(InsertCallNodes::transform);
        manager.add_pass(InsertPhi::transform);
        manager.add_pass(MakeSSA::transform);
        // manager.add_pass(RemoveRedundantCalls::transform);

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
        // manager.add_pass(RemoveRedundantCalls::transform);

        let mut graph = make_even_fib();
        manager.apply(&mut graph);

        write_graph(&graph, "manager.dot")
    }
}
