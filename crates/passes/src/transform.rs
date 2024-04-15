//! Transformations on the IR.
//! These transformations are generally meant to be ran once,
//! to transform between different levels of abstraction.

mod braun_et_al;
mod insert_call;
mod insert_func;
mod insert_phi;
mod lower_to_fsm;
mod make_ssa;
mod nonblocking;
mod explicit_return;
mod fix_branch;
mod rename_variables;

pub use fix_branch::FixBranch;
pub use explicit_return::ExplicitReturn;
pub use braun_et_al::BraunEtAl;
pub use insert_call::InsertCallNodes;
pub use insert_func::InsertFuncNodes;
pub use insert_phi::InsertPhi;
pub use lower_to_fsm::LowerToFsm;
pub use make_ssa::MakeSSA;
pub use nonblocking::Nonblocking;
pub use rename_variables::RenameVariables;
