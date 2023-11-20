pub mod assign;
pub mod branch;
pub mod call;
pub mod func;
pub mod term;
use std::any::Any;

pub use assign::*;
pub use branch::*;
pub use call::*;
pub use func::*;
pub use term::*;

use crate::expr::VarExpr;

pub trait ReadsVariables {
    fn read_vars(&mut self) -> Vec<&mut VarExpr> {
        vec![]
    }
}

pub trait WroteVariables {
    fn wrote_vars(&self) -> Vec<&VarExpr> {
        vec![]
    }
}

#[derive(Clone, PartialEq)]
pub enum Node {
    Assign(AssignNode),
    Branch(BranchNode),
    Yield(TermNode),
    Return(TermNode),
    Func(FuncNode),
    Call(CallNode),
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Node::Assign(n) => write!(f, "{}", n),
            Node::Branch(n) => write!(f, "{}", n),
            Node::Yield(n) => write!(f, "yield{}", n),
            Node::Return(n) => write!(f, "return{}", n),
            Node::Func(n) => write!(f, "{}", n),
            Node::Call(n) => write!(f, "{}", n),
        }
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub trait NodeLike: ReadsVariables + WroteVariables + std::fmt::Display + Any {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn as_any(&self) -> &dyn Any;
    fn filter(value: &Box<dyn NodeLike>) -> bool
    where
        Self: Sized,
    {
        let any = value.as_any();
        match any.downcast_ref::<Self>() {
            Some(_) => true,
            None => false,
        }
    }
    fn concrete(value: &Box<dyn NodeLike>) -> Option<&Self>
    where
        Self: Sized,
    {
        let any = value.as_any();
        match any.downcast_ref::<Self>() {
            Some(inner) => Some(inner),
            None => None,
        }
    }
}

impl<T: ReadsVariables + WroteVariables + std::fmt::Display + Any> NodeLike for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::expr::*;

    use super::*;

    #[test]
    fn dynamic_vec() {
        let mut vec: Vec<Box<dyn NodeLike>> = vec![];
        vec.push(Box::new(AssignNode {
            lvalue: VarExpr::new("a"),
            rvalue: Expr::Int(IntExpr::new(123)),
        }));
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
                    println!("rvalue {}", rvalue);
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
                    println!("rvalue {}", rvalue);
                }
                None => {}
            }
        }

        println!("before retain");
        for value in &vec {
            println!("{}", value);
        }

        for value in &vec {
            if let Some(assign) = AssignNode::concrete(value) {
                println!("Yes {} = {}", assign.lvalue, assign.rvalue);
            } else {
                println!("No {}", value);
            }
        }

        vec.retain(AssignNode::filter);

        println!("after retain");
        for value in &vec {
            println!("{}", value);
        }
    }
}
