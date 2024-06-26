use crate::*;

#[derive(Default)]
pub struct PassManager {
    passes: Vec<fn(&mut CFG) -> TransformResultType>,
    result: TransformResultType,
    log: bool,
    prefix: String,
    write: bool,
}

impl PassManager {
    /// Takes a transform constructor and adds it to the manager
    pub fn add_pass(&mut self, pass: fn(&mut CFG) -> TransformResultType) {
        self.passes.push(pass);
    }

    /// Create a logging pass manager
    pub fn log() -> Self {
        Self {
            passes: vec![],
            result: Default::default(),
            log: true,
            prefix: "".into(),
            write: false,
        }
    }

    /// Create a debug manager
    pub fn debug(prefix: String) -> Self {
        Self {
            passes: vec![],
            result: Default::default(),
            log: true,
            prefix,
            write: false,
        }
    }
}

impl PassManager {
    fn log_pass(&self, result: &TransformResultType) {
        println!("{}", result);
    }
}

impl BasicTransform for PassManager {
    #[track_caller]
    fn apply(&mut self, graph: &mut CFG) -> &TransformResultType {
        if self.write {
            graph.write_dot(&format!("{}_0", self.prefix));
        }
        if self.log {
            println!("Pass Manager at {}", std::panic::Location::caller());
        }
        for (i, pass) in self.passes.iter().enumerate() {
            let result = pass(graph);
            self.result.elapsed_time += result.elapsed_time;
            self.result.did_work |= result.did_work;

            if self.write {
                graph.write_dot(&format!("{}_{}_{}", self.prefix, i + 1, &result.name));
            }
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

        // graph.write_dot("manager.dot")
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

        // graph.write_dot("manager.dot")
    }
}
