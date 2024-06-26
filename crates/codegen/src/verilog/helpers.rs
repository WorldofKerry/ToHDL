use tohdl_ir::expr::VarExpr;
use vast::v05::ast::{self as v, Sequential};

use super::{module::Context, SingleStateLogic};

/// Creates memories and variables stored in reg
fn create_reg_defs(context: &Context) -> Vec<v::Stmt> {
    (0..context.memories.count)
        .map(|i| {
            v::Stmt::new_decl(v::Decl::new_int(&format!(
                "{}{}",
                context.memories.prefix, i
            )))
        })
        .chain(std::iter::once(v::Stmt::new_decl(v::Decl::new_reg(
            &format!("{}", context.states.variable),
            32,
        ))))
        .collect()
}

// Creates localparams for states
fn create_state_defs(case_count: usize, context: &Context) -> Vec<v::Stmt> {
    (0..case_count)
        .map(|i| {
            v::Stmt::RawStr(format!(
                "localparam {}{} = {};",
                context.states.prefix, i, i
            ))
        })
        .chain(vec![
            v::Stmt::RawStr(format!(
                "localparam {} = {};",
                context.states.start, case_count
            )),
            v::Stmt::RawStr(format!(
                "localparam {} = {};",
                context.states.done,
                case_count + 1
            )),
        ])
        .collect()
}

/// ```verilog
/// always_ff @(posedge clock or posedge reset) begin
///     // body
/// end
/// ````
fn new_create_posedge_clock(context: &Context, body: Vec<v::Sequential>) -> v::ParallelProcess {
    let clock_event = Sequential::Event(
        v::EventTy::Posedge,
        v::Expr::new_ref(context.signals.clock.to_string()),
    );

    let mut always_ff = v::ParallelProcess::new_always();
    always_ff.set_event(clock_event);

    for b in body {
        always_ff.add_seq(b);
    }

    always_ff
}

fn var_to_ref(var: &VarExpr) -> v::Expr {
    v::Expr::new_ref(var.to_string())
}

/// ```verilog
/// if (start) begin
///     // start logic
/// end else if (reset) begin
///     // reset logic
/// end else begin
///     // fsm body
/// end
fn new_create_start_ifelse(context: &Context, fsm_body: Vec<v::Sequential>) -> v::SequentialIfElse {
    let mut ifelse = v::SequentialIfElse::new(var_to_ref(&context.signals.start));
    ifelse.add_seq(v::Sequential::new_nonblk_assign(
        var_to_ref(&context.signals.valid),
        v::Expr::Int(0),
    ));
    ifelse.add_seq(v::Sequential::new_nonblk_assign(
        var_to_ref(&context.signals.done),
        v::Expr::Int(0),
    ));
    for (i, input) in context.io.inputs.iter().enumerate() {
        ifelse.add_seq(v::Sequential::new_nonblk_assign(
            v::Expr::new_ref(format!("{}{}", context.memories.prefix, i)),
            v::Expr::new_ref(input.to_string()),
        ));
    }
    ifelse.add_seq(v::Sequential::new_nonblk_assign(
        v::Expr::new_ref(context.states.variable.to_string()),
        v::Expr::new_ref(&format!("{}0", context.states.prefix)),
    ));

    let mut reset = {
        let mut always_ff = v::SequentialIfElse::new(var_to_ref(&context.signals.reset));
        always_ff.add_seq(v::Sequential::new_nonblk_assign(
            v::Expr::new_ref(context.states.variable.to_string()),
            v::Expr::new_ref(context.states.start.to_string()),
        ));
        always_ff.add_seq(v::Sequential::new_nonblk_assign(
            v::Expr::new_ref(context.signals.valid.to_string()),
            v::Expr::Int(0),
        ));
        always_ff.add_seq(v::Sequential::new_nonblk_assign(
            var_to_ref(&context.signals.done),
            v::Expr::Int(0),
        ));
        always_ff
    };

    let mut elsee = v::SequentialIfElse::default();
    for stmt in fsm_body {
        elsee.add_seq(stmt);
    }
    reset.set_else(elsee);
    ifelse.set_else(reset);
    ifelse
}

/// ```verilog
/// if (ready || ~valid) begin
///     // case
/// end
fn new_create_fsm(context: &Context, case: v::Case) -> v::SequentialIfElse {
    let mut ready_or_invalid = v::SequentialIfElse::new(v::Expr::new_logical_or(
        v::Expr::new_ref(context.signals.ready.to_string()),
        v::Expr::new_not(v::Expr::new_ref(context.signals.valid.to_string())),
    ));
    ready_or_invalid.add_seq(v::Sequential::new_nonblk_assign(
        v::Expr::new_ref(context.signals.valid.to_string()),
        v::Expr::Int(0),
    ));
    ready_or_invalid.add_seq(v::Sequential::new_case(case));
    ready_or_invalid
}

pub fn new_create_module(states: Vec<SingleStateLogic>, context: &Context) -> v::Module {
    let memories = create_reg_defs(context);
    let mut case = v::Case::new(v::Expr::new_ref(context.states.variable.to_string()));
    let case_count = {
        let done = create_done_state(context);
        let cases = create_states(states, context);
        case.add_branch(done);
        let case_count = cases.len();
        for c in cases {
            case.add_branch(c);
        }
        case_count
    };
    let state_defs = create_state_defs(case_count, context);
    let fsm = new_create_posedge_clock(
        context,
        vec![v::Sequential::from(new_create_start_ifelse(
            context,
            vec![v::Sequential::IfElse(new_create_fsm(context, case))],
        ))],
    );
    let body = vec![]
        .into_iter()
        .chain(state_defs)
        .chain(memories)
        .chain(std::iter::once(v::Stmt::from(fsm)));

    let mut module = v::Module::new(&context.name);
    for input in context.io.inputs.iter().chain(context.signals.inputs()) {
        module.add_input(&format!("{}", input), input.size as u64);
    }
    for output in context.signals.outputs() {
        module.add_output_reg(&format!("{}", output), output.size as u64);
    }
    for i in 0..context.io.output_count {
        module.add_output_reg(&format!("{}{}", context.io.output_prefix, i), 32);
    }
    for stmt in body {
        module.add_stmt(stmt);
    }
    module
}

fn create_states(states: Vec<SingleStateLogic>, context: &Context) -> Vec<v::CaseBranch> {
    let mut cases = vec![];
    for (i, state) in states.into_iter().enumerate() {
        let mut branch =
            v::CaseBranch::new(v::Expr::Ref(format!("{}{}", context.states.prefix, i)));
        branch.body = state.body;
        cases.push(branch);
    }
    cases
}

fn create_done_state(context: &Context) -> v::CaseBranch {
    let mut branch = v::CaseBranch::new(v::Expr::Ref(context.states.done.to_owned()));
    branch.add_seq(v::Sequential::new_nonblk_assign(
        var_to_ref(&context.signals.done),
        v::Expr::Int(1),
    ));
    branch.add_seq(v::Sequential::new_nonblk_assign(
        var_to_ref(&context.signals.valid),
        v::Expr::Int(1),
    ));
    branch.add_seq(v::Sequential::new_nonblk_assign(
        v::Expr::new_ref(context.states.variable.to_string()),
        v::Expr::new_ref(&format!("{}", context.states.start)),
    ));
    branch
}

#[cfg(test)]
mod test {
    use tohdl_passes::{
        manager::PassManager,
        optimize::RemoveUnreadVars,
        transform::{BraunEtAl, InsertCallNodes, InsertFuncNodes, Nonblocking},
        BasicTransform,
    };
    use vast::v05::ast::CaseBranch;

    use crate::{tests::make_odd_fib, verilog::SingleStateLogic};

    use super::*;

    #[test]
    fn main() {
        let result = create_reg_defs(&Default::default());
        // println!("{:?}", result);
    }
}
