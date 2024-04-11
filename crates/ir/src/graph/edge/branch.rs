use super::Edge;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BranchEdge {
    pub condition: bool,
}

impl BranchEdge {
    pub fn new(condition: bool) -> Self {
        BranchEdge { condition }
    }
}

impl Edge for BranchEdge {}

impl std::fmt::Display for BranchEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.condition)
    }
}
