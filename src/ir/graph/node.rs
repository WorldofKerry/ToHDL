mod assign;
mod branch;
pub use assign::*;
pub use branch::*;

#[derive(Debug, Clone)]
pub enum Node {
    Assign(AssignNode),
}
