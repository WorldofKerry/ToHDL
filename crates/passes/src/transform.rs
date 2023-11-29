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

pub use braun_et_al::BraunEtAl;
pub use insert_call::InsertCallNodes;
pub use insert_func::InsertFuncNodes;
pub use insert_phi::InsertPhi;
pub use lower_to_fsm::LowerToFsm;
pub use make_ssa::MakeSSA;
pub use nonblocking::Nonblocking;
