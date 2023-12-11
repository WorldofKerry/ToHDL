use tohdl_ir::expr::VarExpr;
use vast::{
    util::pretty_print::PrettyPrint,
    v17::ast::{self as v, Sequential},
};

use super::{module::Context, SingleStateLogic};

/// Creates memories and variables stored in reg
fn create_reg_defs(context: &Context) -> Vec<v::Stmt> {
    (0..context.memories.count)
        .map(|i| {
            v::Stmt::new_decl(v::Decl::new_logic(
                &format!("{}{}", context.memories.prefix, i),
                32,
            ))
        })
        .chain(std::iter::once(v::Stmt::new_decl(v::Decl::new_logic(
            &format!("{}", context.states.variable),
            32,
        ))))
        .collect()
}

// Creates localparams for states
fn create_state_defs(case_count: usize, context: &Context) -> Vec<v::Stmt> {
    (0..case_count)
        .map(|i| {
            v::Stmt::new_rawstr(format!(
                "localparam {}{} = {};",
                context.states.prefix, i, i
            ))
        })
        .chain(vec![
            v::Stmt::new_rawstr(format!(
                "localparam {} = {};",
                context.states.start, case_count
            )),
            v::Stmt::new_rawstr(format!(
                "localparam {} = {};",
                context.states.done,
                case_count + 1
            )),
        ])
        .collect()
}

/// ```verilog
/// always_ff @(posedge clock) begin
///     // body
/// end
/// ````
fn new_create_posedge_clock(
    context: &Context,
    body: Vec<v::Sequential>,
) -> vast::v17::ast::ParallelProcess {
    let clock_event = Sequential::Event(
        v::EventTy::Posedge,
        v::Expr::new_ref(context.signals.clock.to_string()),
    );

    let mut always_ff = v::ParallelProcess::new_always_ff();
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
///     mem_x <= ...;
///     ...
///     state <= 0;
/// end else begin
///     // fsm body
/// end
fn new_create_start_ifelse(
    context: &Context,
    fsm_body: Vec<v::Sequential>,
) -> vast::v17::ast::SequentialIfElse {
    let mut ifelse = v::SequentialIfElse::new(var_to_ref(&context.signals.start));
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
    let mut elsee = v::SequentialIfElse::default();
    for stmt in fsm_body {
        elsee.add_seq(stmt);
    }
    ifelse.set_else(elsee);
    ifelse
}

/// ```verilog
/// if (ready || ~valid) begin
///     // case
/// end
fn new_create_fsm(context: &Context, case: v::Case) -> vast::v17::ast::SequentialIfElse {
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
    let reset = {
        // Reset state on reset
        let event = v::Sequential::Event(
            v::EventTy::Posedge,
            v::Expr::new_ref(context.signals.reset.to_string()),
        );
        let mut always_ff = v::ParallelProcess::new_always_ff();
        always_ff.set_event(event);
        always_ff.add_seq(v::Sequential::new_nonblk_assign(
            v::Expr::new_ref(context.states.variable.to_string()),
            v::Expr::new_ref(context.states.start.to_string()),
        ));
        always_ff.add_seq(v::Sequential::new_nonblk_assign(
            v::Expr::new_ref(context.signals.valid.to_string()),
            v::Expr::Int(0),
        ));
        v::Stmt::from(always_ff)
    };
    let fsm = new_create_posedge_clock(
        context,
        vec![v::Sequential::from(new_create_start_ifelse(
            context,
            vec![v::Sequential::If(new_create_fsm(context, case))],
        ))],
    );
    let done = v::Parallel::ParAssign(
        var_to_ref(&context.signals.done),
        v::Expr::new_ref(format!(
            "{} == {}",
            context.states.variable, context.states.done
        )),
    );
    let body = vec![]
        .into_iter()
        .chain(state_defs)
        .chain(memories)
        .chain(std::iter::once(v::Stmt::from(done)))
        .chain(std::iter::once(reset))
        .chain(std::iter::once(v::Stmt::from(fsm)));

    let mut module = v::Module::new(&context.name);
    for input in context.io.inputs.iter().chain(context.signals.inputs()) {
        module.add_input(&format!("{}", input), input.size as u64);
    }
    for output in context.signals.outputs() {
        module.add_output(&format!("{}", output), output.size as u64);
    }
    for i in 0..context.io.output_count {
        module.add_output(&format!("{}{}", context.io.output_prefix, i), 32);
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
    // branch.add_seq(v::Sequential::new_nonblk_assign(
    //     v::Expr::new_ref(context.signals.done.to_string()),
    //     v::Expr::Int(1),
    // ));
    let mut ifelse = v::SequentialIfElse::new(var_to_ref(&context.signals.ready));
    ifelse.add_seq(v::Sequential::new_nonblk_assign(
        v::Expr::new_ref(context.states.variable.to_string()),
        v::Expr::new_ref(&format!("{}", context.states.start)),
    ));
    branch.add_seq(ifelse);
    branch
}

#[cfg(test)]
mod test {
    use tohdl_passes::{
        manager::PassManager,
        optimize::RemoveUnreadVars,
        transform::{BraunEtAl, InsertCallNodes, InsertFuncNodes, Nonblocking},
        Transform,
    };
    use vast::v05::ast::CaseBranch;

    use crate::{tests::make_odd_fib, verilog::SingleStateLogic};

    use super::*;

    #[test]
    fn main() {
        let result = create_reg_defs(&Default::default());
        println!("{:?}", result);
    }
}
