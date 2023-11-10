#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Edge {
    Branch(bool),
    Extern,
    None,
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Edge::Branch(b) => write!(f, "{}", b),
            Edge::Extern => write!(f, "extern"),
            Edge::None => write!(f, ""),
        }
    }
}
