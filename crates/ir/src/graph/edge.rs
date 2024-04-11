use std::{any::Any, fmt::Debug, fmt::Display};

use downcast_rs::{impl_downcast, Downcast};

pub trait EdgeTrait: Debug + Display + Any + dyn_clone::DynClone + Downcast {}
dyn_clone::clone_trait_object!(EdgeTrait);
impl_downcast!(EdgeTrait);

impl<T> From<T> for Box<dyn EdgeTrait>
where
    T: EdgeTrait,
{
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NoneEdge;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BranchEdge {
    pub condition: bool,
}

impl BranchEdge {
    pub fn new(condition: bool) -> Self {
        BranchEdge { condition }
    }
}

impl EdgeTrait for BranchEdge {}
impl EdgeTrait for NoneEdge {}

impl std::fmt::Display for BranchEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.condition)
    }
}

impl std::fmt::Display for NoneEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "")
    }
}
