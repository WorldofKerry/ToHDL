pub mod assign;
pub mod branch;
pub mod call;
pub mod func;
pub mod term;
pub use assign::*;
pub use branch::*;
pub use call::*;
pub use func::*;
pub use term::*;

use crate::expr::VarExpr;

pub trait ReadsVariables {
    fn read_vars(&mut self) -> Vec<&mut VarExpr>;
}

pub trait WroteVariables {
    fn wrote_vars(&self) -> Vec<&VarExpr>;
}

#[derive(Clone, PartialEq)]
pub enum Node {
    Assign(AssignNode),
    Branch(BranchNode),
    Yield(TermNode),
    Return(TermNode),
    Func(FuncNode),
    Call(CallNode),
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Node::Assign(n) => write!(f, "{}", n),
            Node::Branch(n) => write!(f, "{}", n),
            Node::Yield(n) => write!(f, "yield{}", n),
            Node::Return(n) => write!(f, "return{}", n),
            Node::Func(n) => write!(f, "{}", n),
            Node::Call(n) => write!(f, "{}", n),
        }
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
