use std::any::Any;

pub trait DispatchableBounds: Any + dyn_clone::DynClone {}

pub trait Dispatchable: DispatchableBounds {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn downcastable(node: &Box<dyn Dispatchable>) -> bool
    where
        Self: Sized,
    {
        let any = node.as_any();
        any.downcast_ref::<Self>().is_some()
    }

    fn concrete(value: &Box<dyn Dispatchable>) -> Option<&Self>
    where
        Self: Sized,
    {
        let any = value.as_any();
        match any.downcast_ref::<Self>() {
            Some(inner) => Some(inner),
            None => None,
        }
    }

    fn concrete_mut(value: &mut Box<dyn Dispatchable>) -> Option<&mut Self>
    where
        Self: Sized,
    {
        let any = value.as_any_mut();
        match any.downcast_mut::<Self>() {
            Some(inner) => Some(inner),
            None => None,
        }
    }
}

dyn_clone::clone_trait_object!(Dispatchable);

impl<T> Dispatchable for T
where
    T: DispatchableBounds,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub trait Operation: Dispatchable {}

#[derive(Clone)]
pub struct AddLike {}

impl Operation for AddLike {}

impl DispatchableBounds for AddLike {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn main() {
        let v: Vec<Box<dyn Operation>> = vec![Box::new(AddLike {})];
        // if let Some(add) = AddLike::concrete(v.get(0)) {

        // }
    }
}
