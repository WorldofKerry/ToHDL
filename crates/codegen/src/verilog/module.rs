use tohdl_ir::expr::VarExpr;
use typed_builder::TypedBuilder;
use vast::v17::ast::{self as v, Sequential};

#[derive(Debug)]
pub struct Signals {
    pub ready: VarExpr,
    pub valid: VarExpr,
    pub start: VarExpr,
    pub done: VarExpr,
    pub clock: VarExpr,
    pub reset: VarExpr,
}

impl Default for Signals {
    fn default() -> Self {
        Self {
            ready: VarExpr::new("ready"),
            valid: VarExpr::new("valid"),
            start: VarExpr::new("start"),
            done: VarExpr::new("done"),
            clock: VarExpr::new("clock"),
            reset: VarExpr::new("reset"),
        }
    }
}

impl Signals {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn values(&self) -> impl Iterator<Item = &VarExpr> {
        self.inputs().chain(self.outputs())
    }

    pub fn inputs(&self) -> impl Iterator<Item = &VarExpr> {
        vec![&self.ready, &self.start, &self.clock, &self.reset].into_iter()
    }

    pub fn outputs(&self) -> impl Iterator<Item = &VarExpr> {
        vec![&self.valid, &self.done].into_iter()
    }
}

#[derive(Default, Debug)]
pub struct Context {
    pub name: String,
    pub io: InputOutput,
    pub signals: Signals,
    pub states: States,
    pub memories: Memories,
}

impl Context {
    pub fn new<S: Into<String>>(name: S, inputs: Vec<VarExpr>, signals: Signals) -> Self {
        Context {
            name: name.into(),
            io: InputOutput::builder().inputs(inputs).build(),
            signals,
            states: States::default(),
            memories: Memories::default(),
        }
    }
}

#[derive(Debug, Default, TypedBuilder)]
pub struct InputOutput {
    pub inputs: Vec<VarExpr>,

    #[builder(default = 0)]
    pub output_count: usize,
    #[builder(default="out_".into())]
    pub output_prefix: String,
}

#[derive(Debug)]
pub struct States {
    pub variable: String,

    pub start: String,
    pub done: String,

    pub prefix: String,

    // Excludes start and stop states
    pub count: usize,
}

impl Default for States {
    fn default() -> Self {
        Self {
            variable: "state".into(),
            start: "state_start".into(),
            done: "state_done".into(),
            prefix: "state_".into(),
            count: 0,
        }
    }
}

#[derive(Debug)]
pub struct Memories {
    pub prefix: String,
    pub count: usize,
}

impl Default for Memories {
    fn default() -> Self {
        Self {
            prefix: "mem_".into(),
            count: 0,
        }
    }
}

#[cfg(test)]
mod test {
    use tohdl_ir::graph::CFG;
    use tohdl_passes::{
        manager::PassManager,
        optimize::RemoveUnreadVars,
        transform::{BraunEtAl, InsertCallNodes, InsertFuncNodes, Nonblocking},
        Transform,
    };
    use vast::v05::ast::CaseBranch;

    use crate::{
        tests::make_odd_fib,
        verilog::{helpers::*, SingleStateLogic},
    };

    use super::*;
    #[test]
    fn demo() {
        let mut module = v::Module::new("foo");
        module.add_input("a", 32);
        let res = module.to_string();
        let exp = r#"module foo (
    input logic [31:0] a
);
endmodule
"#;
        assert_eq!(res, exp);
    }

    #[test]
    fn odd_fib() {
        let graph = make_odd_fib();
        module(graph)
    }

    #[test]
    fn memory() {
        let code = r#"
def memory():
    a = 10
    b = 20
    c = 30
    yield a
    yield c
    yield b
"#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);
        let graph = visitor.get_graph();
        module(graph);
    }

    fn module(mut graph: CFG) {
        let mut manager = PassManager::default();

        manager.add_pass(InsertFuncNodes::transform);
        manager.add_pass(InsertCallNodes::transform);
        manager.add_pass(BraunEtAl::transform);

        manager.apply(&mut graph);

        let mut lower = tohdl_passes::transform::LowerToFsm::default();
        lower.apply(&mut graph);

        let mut states = vec![];

        let signals = Signals::new();
        let mut context = Context::new("fib", graph.get_inputs().cloned().collect(), signals);

        // Write all new subgraphs to files
        for (i, subgraph) in lower.get_subgraphs().iter().enumerate() {
            let mut subgraph = subgraph.clone();
            let max_memory = {
                let mut pass = crate::verilog::UseMemory::default();
                pass.apply(&mut subgraph);
                pass.max_memory()
            };
            Nonblocking::transform(&mut subgraph);
            RemoveUnreadVars::transform(&mut subgraph);
            context.memories.count = std::cmp::max(context.memories.count, max_memory);

            subgraph.write_dot(format!("debug_{}.dot", i).as_str());
            let mut codegen = SingleStateLogic::new(subgraph, i, lower.get_external_funcs(i));
            codegen.apply(&mut context);
            // println!("codegen body {:?}", codegen.body);
            states.push(codegen);
        }

        let body = create_module_body(states, &context);
        let module = create_module(body, &context);
        println!("{}", module);
    }
}
