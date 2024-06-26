mod assign;
mod branch;
mod call;
mod func;
mod term;
mod external;

pub use assign::*;
pub use branch::*;
pub use call::*;
pub use func::*;
pub use term::*;
pub use external::*;

use crate::expr::{Expr, VarExpr};
use std::{any::Any, collections::BTreeMap};

pub trait DataFlow {
    fn referenced_vars(&self) -> Vec<&VarExpr> {
        vec![]
    }
    fn declared_vars(&self) -> Vec<&VarExpr> {
        vec![]
    }
    fn referenced_vars_mut(&mut self) -> Vec<&mut VarExpr> {
        vec![]
    }
    fn declared_vars_mut(&mut self) -> Vec<&mut VarExpr> {
        vec![]
    }
    fn referenced_exprs_mut(&mut self) -> Vec<&mut Expr> {
        vec![]
    }
    /// Tell node to undefine a variable
    /// Returns true if node should be removed, false otherwise
    fn undefine_var(&mut self, _var: &VarExpr) -> bool {
        panic!("Must be overwritten");
    }
    fn defined_vars(&self) -> BTreeMap<&VarExpr, &Expr> {
        Default::default()
    }
}

pub trait Node: DataFlow + std::fmt::Display + Any + dyn_clone::DynClone {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn downcastable(node: &Box<dyn Node>) -> bool
    where
        Self: Sized,
    {
        let any = node.as_any();
        any.downcast_ref::<Self>().is_some()
    }

    /// Gets underlying type of node
    fn concrete(value: &Box<dyn Node>) -> Option<&Self>
    where
        Self: Sized,
    {
        let any = value.as_any();
        match any.downcast_ref::<Self>() {
            Some(inner) => Some(inner),
            None => None,
        }
    }

    /// Gets mutable underlying type of node
    fn concrete_mut(value: &mut Box<dyn Node>) -> Option<&mut Self>
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

dyn_clone::clone_trait_object!(Node);

impl<T> Node for T
where
    T: DataFlow + std::fmt::Display + Any + dyn_clone::DynClone,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<T> From<T> for Box<dyn Node>
where
    T: Node,
{
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::expr::*;

    use super::*;

    #[test]
    fn dynamic_vec() {
        let mut vec: Vec<Box<dyn Node>> = vec![];
        vec.push(
            AssignNode {
                lvalue: VarExpr::new("a"),
                rvalue: Expr::Int(IntExpr::new(123)),
            }
            .into(),
        );
        vec.push(Box::new(BranchNode {
            cond: Expr::BinOp(
                Box::new(Expr::Var(VarExpr::new("a"))),
                Operator::Lt,
                Box::new(Expr::Int(IntExpr::new(456))),
            ),
        }));
        vec.push(Box::new(BranchNode {
            cond: Expr::BinOp(
                Box::new(Expr::Var(VarExpr::new("a"))),
                Operator::Lt,
                Box::new(Expr::Int(IntExpr::new(456))),
            ),
        }));
        vec.push(Box::new(AssignNode {
            lvalue: VarExpr::new("a"),
            rvalue: Expr::Int(IntExpr::new(999)),
        }));
        vec.push(Box::new(BranchNode {
            cond: Expr::BinOp(
                Box::new(Expr::Var(VarExpr::new("a"))),
                Operator::Lt,
                Box::new(Expr::Int(IntExpr::new(456))),
            ),
        }));

        for value in &mut vec {
            let any = value.as_any_mut();
            match any.downcast_mut::<AssignNode>() {
                Some(&mut AssignNode {
                    lvalue: _,
                    ref mut rvalue,
                }) => {
                    // println!("rvalue {}", rvalue);
                    *rvalue = Expr::Int(IntExpr::new(420));
                }
                None => {}
            }
        }

        for value in &mut vec {
            let any = value.as_any_mut();
            match any.downcast_mut::<AssignNode>() {
                Some(&mut AssignNode {
                    lvalue: _,
                    ref mut rvalue,
                }) => {
                    // println!("rvalue {}", rvalue);
                }
                None => {}
            }
        }

        // println!("before retain");
        for value in &vec {
            // println!("{}", value);
        }

        for value in &mut vec {
            if let Some(assign) = AssignNode::concrete_mut(value) {
                // println!("Yes {} = {}", assign.lvalue, assign.rvalue);
                assign.rvalue = Expr::Int(IntExpr::new(9000))
            } else {
                // println!("No {}", value);
            }
        }

        vec.retain(AssignNode::downcastable);

        // println!("after retain");
        for value in &vec {
            // println!("{}", value);
        }
    }

    #[test]
    fn min_repoducible_example() {
        let lvalue = VarExpr::new("a");
        let rvalue = Expr::Int(IntExpr::new(123));

        let value: Box<dyn Node> = Box::new(AssignNode { lvalue, rvalue });
        if let Some(assign) = AssignNode::concrete(&value) {
            // println!("Yes {} = {}", assign.lvalue, assign.rvalue);
        } else {
            // println!("No {}", value);
        }
    }
}
