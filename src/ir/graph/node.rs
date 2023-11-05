mod assign;
mod branch;
mod term;
pub use assign::*;
pub use branch::*;
pub use term::*;

#[derive(Clone)]
pub enum Node {
    Assign(AssignNode),
    Branch(BranchNode),
    Yield(TermNode),
    Return(TermNode),
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Node::Assign(n) => write!(f, "{}", n),
            Node::Branch(n) => write!(f, "{}", n),
            Node::Yield(n) => write!(f, "{}", n),
            Node::Return(n) => write!(f, "{}", n),
        }
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Node::Assign(n) => write!(f, "{}", n),
            Node::Branch(n) => write!(f, "{}", n),
            Node::Yield(n) => write!(f, "{}", n),
            Node::Return(n) => write!(f, "{}", n),
        }
    }
}
