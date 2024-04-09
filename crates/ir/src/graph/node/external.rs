use super::DataFlow;

#[derive(Clone, PartialEq)]
pub struct ExternalNode {
    pub name: String,
}

impl std::fmt::Display for ExternalNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "external({})", self.name)
    }
}

impl DataFlow for ExternalNode {
}
