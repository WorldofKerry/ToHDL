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
            ready: VarExpr::builder().name("__ready").size(1).build(),
            valid: VarExpr::builder().name("__valid").size(1).build(),
            start: VarExpr::builder().name("__start").size(1).build(),
            done: VarExpr::builder().name("__done").size(1).build(),
            clock: VarExpr::builder().name("__clock").size(1).build(),
            reset: VarExpr::builder().name("__reset").size(1).build(),
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
    #[builder(default="__output_".into())]
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
            variable: "__state".into(),
            start: "__state_start".into(),
            done: "__state_done".into(),
            prefix: "__state_".into(),
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
        transform::{
            BraunEtAl, ExplicitReturn, FixBranch, InsertCallNodes, InsertFuncNodes, Nonblocking,
        },
        Transform,
    };
    use vast::v05::ast::CaseBranch;

    use crate::{
        tests::make_odd_fib,
        verilog::{
            graph_to_verilog, helpers::*, memory::RemoveLoadsEtc, RemoveAssignNodes,
            SingleStateLogic,
        },
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
        let res = graph_to_verilog(graph);
        println!("{res}")
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
        let res = graph_to_verilog(graph);
        // println!("{res}")
    }

    #[test]
    fn range() {
        let code = r#"
def p2vrange(start: int, stop: int, step: int) -> int:
    """
    Simplified version of Python's built-in range function
    """
    while start < stop:
        yield start
        start += step
"#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);
        let graph = visitor.get_graph();
        let res = graph_to_verilog(graph);
        println!("{res}")
    }

    #[test]
    fn multiplier() {
        let code = r#"
def multiplier(multiplicand: int, multiplier: int) -> int:
    product = 0
    while multiplier > 0:
        product += multiplicand
        multiplier -= 1
    yield product
"#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);
        let graph = visitor.get_graph();
        let res = graph_to_verilog(graph);
        println!("{res}")
    }

    #[test]
    fn adder() {
        let code = r#"
def adder(a: int, b: int) -> int:
    yield a + b
"#;
        let visitor = tohdl_frontend::AstVisitor::from_text(code);
        let graph = visitor.get_graph();
        let res = graph_to_verilog(graph);
        println!("{res}")
    }
}
