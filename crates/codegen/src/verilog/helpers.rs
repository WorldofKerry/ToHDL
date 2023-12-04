use tohdl_ir::expr::VarExpr;
use vast::v17::ast::{self as v, Sequential};

use super::{module::Context, SingleStateLogic};

/// Creates memories and variables stored in reg
pub fn create_reg_defs(context: &Context) -> Vec<v::Stmt> {
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
pub fn create_state_defs(case_count: usize, context: &Context) -> Vec<v::Stmt> {
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

/// Create FSM using posedge always block
pub fn create_fsm(case: v::Case, context: &Context) -> v::Stmt {
    let mut ready_or_invalid = v::SequentialIfElse::new(v::Expr::new_logical_or(
        v::Expr::new_ref(context.signals.ready.to_string()),
        v::Expr::new_not(v::Expr::new_ref(context.signals.valid.to_string())),
    ));
    ready_or_invalid.add_seq(v::Sequential::new_case(case));
    let event = Sequential::Event(
        v::EventTy::Posedge,
        v::Expr::new_ref(context.signals.clock.to_string()),
    );
    let mut always_ff = v::ParallelProcess::new_always_ff();
    always_ff.set_event(event);
    always_ff.add_seq(ready_or_invalid);
    let stmt = v::Stmt::from(always_ff);
    stmt
}

pub fn create_states(states: Vec<SingleStateLogic>, context: &Context) -> Vec<v::CaseBranch> {
    let mut cases = vec![];
    for (i, state) in states.into_iter().enumerate() {
        let mut branch =
            v::CaseBranch::new(v::Expr::Ref(format!("{}{}", context.states.prefix, i)));
        branch.body = state.body;
        cases.push(branch);
    }
    cases
}

pub fn create_module_body(states: Vec<SingleStateLogic>, context: &Context) -> Vec<v::Stmt> {
    let memories = create_reg_defs(context);
    let mut case = v::Case::new(v::Expr::new_ref("state"));
    let case_count = {
        let entry = create_start_state(context);
        let cases = create_states(states, context);
        case.add_branch(entry);
        let case_count = cases.len();
        for c in cases {
            case.add_branch(c);
        }
        case_count
    };
    let state_defs = create_state_defs(case_count, context);
    let fsm = create_fsm(case, context);
    vec![]
        .into_iter()
        .chain(memories)
        .chain(state_defs)
        .chain(std::iter::once(fsm))
        .collect()
}

pub fn create_start_state(context: &Context) -> v::CaseBranch {
    let mut branch = v::CaseBranch::new(v::Expr::Ref(context.states.start.to_owned()));
    for (i, input) in context.io.inputs.iter().enumerate() {
        branch.add_seq(v::Sequential::new_nonblk_assign(
            v::Expr::new_ref(format!("{}{}", context.memories.prefix, i)),
            v::Expr::new_ref(input.to_string()),
        ));
    }
    {
        let mut ifelse =
            v::SequentialIfElse::new(v::Expr::new_ref(context.signals.start.to_string()));
        ifelse.add_seq(v::Sequential::new_nonblk_assign(
            v::Expr::new_ref(context.states.variable.to_string()),
            v::Expr::new_ref(&format!("{}0", context.states.prefix)),
        ));
        branch.add_seq(v::Sequential::If(ifelse));
    }
    branch
}

pub fn create_module(body: Vec<v::Stmt>, context: &Context) -> v::Module {
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
