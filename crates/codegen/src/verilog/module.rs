use tohdl_ir::expr::VarExpr;
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
    pub inputs: Vec<VarExpr>,
    pub outputs: Vec<VarExpr>,
    pub signals: Signals,
}

impl Context {
    pub fn new<S: Into<String>>(
        name: S,
        inputs: Vec<VarExpr>,
        outputs: Vec<VarExpr>,
        signals: Signals,
    ) -> Self {
        Context {
            inputs,
            outputs,
            name: name.into(),
            signals,
        }
    }
}

pub fn make_module(case: v::Case, context: &Context) -> v::Module {
    let mut module = v::Module::new("myname");
    for input in context.inputs.iter().chain(context.signals.inputs()) {
        module.add_input(&format!("{}", input), input.size as u64);
    }
    for output in context.outputs.iter().chain(context.signals.outputs()) {
        module.add_output(&format!("{}", output), output.size as u64);
    }
    module
}

#[cfg(test)]
mod test {
    use tohdl_passes::{
        manager::PassManager,
        optimize::RemoveUnreadVars,
        transform::{BraunEtAl, InsertCallNodes, InsertFuncNodes, Nonblocking},
        Transform,
    };

    use crate::tests::make_odd_fib;

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
    fn module() {
        let signals = Signals::new();
        let context = Context::new("fib", vec![VarExpr::new("n")], vec![], signals);
        let case = v::Case::new(v::Expr::Ref("state".into()));
        let module = make_module(case, &context);
        println!("{}", module);
    }
}
