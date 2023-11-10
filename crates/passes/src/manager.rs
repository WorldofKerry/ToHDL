use crate::*;

struct PassManager {
    passes: Vec<fn(&mut DiGraph) -> ()>,
}

impl PassManager {
    pub fn new() -> Self {
        Self { passes: vec![] }
    }

    // Takes a transform constructor and adds it to the manager
    pub fn add_pass(&mut self, pass: fn(&mut DiGraph) -> ()) {
        self.passes.push(pass);
    }

    pub fn transform(&mut self, graph: &mut DiGraph) {
        for pass in &self.passes {
            pass(graph);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    #[test]
    fn main() {
        let mut manager = PassManager::new();
        // manager.add_pass(Box::new(insert_func::InsertFuncNodes {}));
        // manager.add_pass(Box::new(insert_call::InsertCallNodes {}));
        // manager.add_pass(Box::new(insert_phi::InsertPhi {}));
        // manager.add_pass(Box::new(make_ssa::MakeSSA::new()));

        manager.add_pass(insert_func::InsertFuncNodes::transform);
        // manager.add_pass(|| Box::new(insert_call::InsertCallNodes {}));

        let mut graph = make_range();
        manager.transform(&mut graph);

        write_graph(&graph, "manager.dot")
    }
}
