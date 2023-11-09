use crate::*;

struct PassManager {
    passes: Vec<Box<dyn Transform>>,
}

impl PassManager {
    pub fn new() -> Self {
        Self { passes: vec![] }
    }

    pub fn add_pass(&mut self, pass: Box<dyn Transform>) {
        self.passes.push(pass);
    }

    pub fn transform(&mut self, graph: &mut DiGraph) {
        for pass in &mut self.passes {
            pass.transform(graph);
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
        manager.add_pass(Box::new(insert_func::InsertFuncNodes {}));
        manager.add_pass(Box::new(insert_call::InsertCallNodes {}));
        manager.add_pass(Box::new(insert_phi::InsertPhi {}));
        manager.add_pass(Box::new(make_ssa::MakeSSA::new()));

        let mut graph = make_range();
        manager.transform(&mut graph);

        write_graph(&graph, "manager.dot")
    }
}
