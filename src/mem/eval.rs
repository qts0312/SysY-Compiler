//! # Eval
//! 
//! In this file, we define a trait for evaluating a expression and implement it for AST.
//! 

use crate::ast::*;
use crate::mem::scope::{ Scope, Entry };

pub trait Eval {
    fn evaluate(&self, scope: &Scope) -> Option<i32>;
}

impl Eval for Exp {
    fn evaluate(&self, scope: &Scope) -> Option<i32> {
        self.lor.evaluate(scope)
    }
}

impl Eval for LVal {
    fn evaluate(&self, scope: &Scope) -> Option<i32> {
        match scope.value(&self.id) {
            Entry::Const(num) => Some(*num),
            Entry::Value(_) => None,
        }
    }
}

impl Eval for PrimaryExp {
    fn evaluate(&self, scope: &Scope) -> Option<i32> {
        match self {
            Self::Exp(exp) => exp.evaluate(scope),
            Self::LVal(lval) => lval.evaluate(scope),
            Self::Num(num) => Some(*num),
        }
    }
}

impl Eval for UnaryExp {
    fn evaluate(&self, scope: &Scope) -> Option<i32> {
        match self {
            Self::Primary(primary) => primary.evaluate(scope),
            Self::Unary(op, unary) => {
                match op {
                    UnaryOp::Pos => unary.evaluate(scope),
                    UnaryOp::Neg => unary.evaluate(scope).map(|num| -num),
                    UnaryOp::Not => unary.evaluate(scope).map(|num| if num == 0 { 1 } else { 0 }),
                }
            }
            Self::Call(_) => None,
        }
    }
}

impl Eval for MulExp {
    fn evaluate(&self, scope: &Scope) -> Option<i32> {
        match self {
            Self::Unary(unary) => unary.evaluate(scope),
            Self::Mul(mul, op, unary) => {
                let left = mul.evaluate(scope)?;
                let right = unary.evaluate(scope)?;
                match op {
                    MulOp::Mul => Some(left * right),
                    MulOp::Div => Some(left / right),
                    MulOp::Mod => Some(left % right),
                }
            }
        }
    }
}

impl Eval for AddExp {
    fn evaluate(&self, scope: &Scope) -> Option<i32> {
        match self {
            Self::Mul(mul) => mul.evaluate(scope),
            Self::Add(add, op, mul) => {
                let left = add.evaluate(scope)?;
                let right = mul.evaluate(scope)?;
                match op {
                    AddOp::Add => Some(left + right),
                    AddOp::Sub => Some(left - right),
                }
            }
        }
    }
}

impl Eval for RelExp {
    fn evaluate(&self, scope: &Scope) -> Option<i32> {
        match self {
            Self::Add(add) => add.evaluate(scope),
            Self::Rel(rel, op, add) => {
                let left = rel.evaluate(scope)?;
                let right = add.evaluate(scope)?;
                match op {
                    RelOp::Lt => Some((left < right) as i32),
                    RelOp::Gt => Some((left > right) as i32),
                    RelOp::Le => Some((left <= right) as i32),
                    RelOp::Ge => Some((left >= right) as i32),
                }
            }
        }
    }
}

impl Eval for EqExp {
    fn evaluate(&self, scope: &Scope) -> Option<i32> {
        match self {
            Self::Rel(rel) => rel.evaluate(scope),
            Self::Eq(eq, op, rel) => {
                let left = eq.evaluate(scope)?;
                let right = rel.evaluate(scope)?;
                match op {
                    EqOp::Eq => Some((left == right) as i32),
                    EqOp::Ne => Some((left != right) as i32),
                }
            }
        }
    }
}

impl Eval for LAndExp {
    fn evaluate(&self, scope: &Scope) -> Option<i32> {
        match self {
            Self::Eq(eq) => eq.evaluate(scope),
            Self::LAnd(land, eq) => {
                let left = land.evaluate(scope)?;
                if left == 0 {
                    Some(0)
                } else {
                    eq.evaluate(scope)
                }
            }
        }
    }
}

impl Eval for LOrExp {
    fn evaluate(&self, scope: &Scope) -> Option<i32> {
        match self {
            Self::LAnd(land) => land.evaluate(scope),
            Self::LOr(lor, land) => {
                let left = lor.evaluate(scope)?;
                if left != 0 {
                    Some(1)
                } else {
                    land.evaluate(scope)
                }
            }
        }
    }
}
