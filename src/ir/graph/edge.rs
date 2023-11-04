pub enum Edge {
    Branch(bool),
    None,
}

impl std::fmt::Debug for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Edge::Branch(b) => write!(f, "{}", b),
            Edge::None => write!(f, ""),
        }
    }
}
