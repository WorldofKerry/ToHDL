use super::Edge;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NoneEdge;

impl Edge for NoneEdge {}

impl std::fmt::Display for NoneEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "")
    }
}
