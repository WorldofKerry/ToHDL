mod none;
mod branch;
pub use none::NoneEdge;
pub use branch::BranchEdge;

use std::{any::Any, fmt::Debug, fmt::Display};

use downcast_rs::{impl_downcast, Downcast};

pub trait Edge: Debug + Display + Any + dyn_clone::DynClone + Downcast {}
dyn_clone::clone_trait_object!(Edge);
impl_downcast!(Edge);

impl<T> From<T> for Box<dyn Edge>
where
    T: Edge,
{
    fn from(value: T) -> Self {
        Box::new(value)
    }
}
