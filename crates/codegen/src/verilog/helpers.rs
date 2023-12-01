use tohdl_ir::expr::VarExpr;
use vast::v17::ast::{self as v, Sequential};

use super::SingleStateLogic;

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

pub fn create_case(states: Vec<SingleStateLogic>) -> v::Case {
    let mut case = v::Case::new(v::Expr::Ref("state".into()));
    for (i, state) in states.into_iter().enumerate() {
        let mut branch = v::CaseBranch::new(v::Expr::Ref(format!("state_{}", i)));
        branch.body = state.body;
        case.add_branch(branch);
    }
    case
}

pub fn create_module_body(states: Vec<SingleStateLogic>) -> Vec<v::Stmt> {
    let memories = create_memories(states.iter().map(|s| s.max_memory).max().unwrap());
    let case = create_case(states);
    let fsm = create_fsm(case);
    vec![]
        .into_iter()
        .chain(memories.into_iter())
        .chain(std::iter::once(fsm))
        .collect()
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
