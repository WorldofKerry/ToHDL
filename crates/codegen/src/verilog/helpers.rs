use tohdl_ir::expr::VarExpr;
use vast::v17::ast::{self as v, Sequential};

use super::{module::Context, SingleStateLogic};

/// Creates memories
pub fn create_memories(max_memory: usize) -> Vec<v::Stmt> {
    (0..max_memory)
        .map(|i| v::Stmt::new_decl(v::Decl::new_logic(&format!("mem_{}", i), 32)))
        .collect()
}

/// Create FSM using posedge always block
pub fn create_fsm(case: v::Case) -> v::Stmt {
    let event = Sequential::Event(v::EventTy::Posedge, v::Expr::Ref("clockkkk".to_string()));
    let mut always_ff = v::ParallelProcess::new_always_ff();
    always_ff.set_event(event);
    always_ff.add_seq(v::Sequential::SeqCase(case));
    let stmt = v::Stmt::from(always_ff);
    stmt
}

pub fn create_case(states: Vec<SingleStateLogic>) -> Vec<v::CaseBranch> {
    let mut cases = vec![];
    for (i, state) in states.into_iter().enumerate() {
        let mut branch = v::CaseBranch::new(v::Expr::Ref(format!("state_{}", i)));
        branch.body = state.body;
        cases.push(branch);
    }
    cases
}

pub fn create_module_body(states: Vec<SingleStateLogic>, context: &Context) -> Vec<v::Stmt> {
    let memories = create_memories(states.iter().map(|s| s.max_memory).max().unwrap());
    let mut case = v::Case::new(v::Expr::new_ref("state"));
    {
        let entry = create_entry_state(context);
        let cases = create_case(states);
        case.add_branch(entry);
        for c in cases {
            case.add_branch(c);
        }
    }
    let fsm = create_fsm(case);
    vec![]
        .into_iter()
        .chain(memories.into_iter())
        .chain(std::iter::once(fsm))
        .collect()
}

pub fn create_entry_state(context: &Context) -> v::CaseBranch {
    let mut branch = v::CaseBranch::new(v::Expr::Ref("state_start".to_owned()));
    for (i, input) in context.io.inputs.iter().enumerate() {
        branch.add_seq(v::Sequential::new_nonblk_assign(
            v::Expr::new_ref(format!("mem_{}", i)),
            v::Expr::new_ref(input.to_string()),
        ));
    }
    branch
}

pub fn create_module(body: Vec<v::Stmt>, context: &Context) -> v::Module {
    let mut module = v::Module::new("myname");
    for input in context.io.inputs.iter().chain(context.signals.inputs()) {
        module.add_input(&format!("{}", input), input.size as u64);
    }
    // for output in context.outputs.iter().chain(context.signals.outputs()) {
    //     module.add_output(&format!("{}", output), output.size as u64);
    // }
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
        let result = create_memories(10);
        println!("{:?}", result);
    }
}
