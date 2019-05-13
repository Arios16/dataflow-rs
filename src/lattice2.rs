use rustc::mir;
use rustc::mir::interpret::ConstValue;
use rustc::mir::{BinOp, Operand, Place, PlaceBase, Rvalue, UnOp};
use rustc::mir::{Local, LocalDecl};
use rustc::ty::TyKind;
use rustc_data_structures::indexed_vec::IndexVec;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignAnalysis {
    Top,
    Bottom,
    Lower,
    Zero,
    Greater,
}

use SignAnalysis::*;
impl SimpleLattice for SignAnalysis {
    fn applies(ty: &TyKind) -> bool {
        match ty {
            &TyKind::Int(_) => true,
            _ => false,
        }
    }

    fn bot() -> Self {
        Bottom
    }

    fn top() -> Self {
        Top
    }

    fn join(op1: &Self, op2: &Self) -> Self {
        if *op1 == *op2 {
            return *op1;
        }
        match (*op1, *op2) {
            (Top, _) | (_, Top) => Top,
            (Bottom, a) => a,
            (a, Bottom) => a,
            _ => Top,
        }
    }

    fn alpha(a: ConstValue) -> Self {
        match a {
            ConstValue::Scalar(scalar) => {
                let x = scalar.to_i32().unwrap();
                if x < 0 {
                    Lower
                } else if x > 0 {
                    Greater
                } else {
                    Zero
                }
            }
            _ => Top,
        }
    }

    fn flow_binop(op: &BinOp, arg1: &Self, arg2: &Self) -> Self {
        match op {
            BinOp::Add => match (arg1, arg2) {
                (Greater, Zero) | (Zero, Greater) | (Greater, Greater) => Greater,
                (Lower, Zero) | (Zero, Lower) | (Lower, Lower) => Lower,
                (Zero, Zero) => Zero,
                _ => Top,
            },
            BinOp::Sub => match (arg1, arg2) {
                (Greater, Zero) | (Zero, Lower) | (Greater, Lower) => Greater,
                (Zero, Greater) | (Lower, Greater) | (Lower, Zero) => Lower,
                (Zero, Zero) => Zero,
                _ => Top,
            },
            BinOp::Mul => match (arg1, arg2) {
                (Zero, _) | (_, Zero) => Zero,
                (Lower, Lower) | (Greater, Greater) => Greater,
                (Greater, Lower) | (Lower, Greater) => Lower,
                _ => Top,
            },
            BinOp::Div => match (arg1, arg2) {
                (Zero, _) => Zero,
                _ => Top,
            },
            BinOp::Rem => *arg1,
            _ => Top,
        }
    }

    fn flow_unop(op: &UnOp, arg: &Self) -> Self {
        match op {
            UnOp::Neg => match arg {
                Zero => Zero,
                Greater => Lower,
                Lower => Greater,
                _ => Top,
            },
            _ => Top,
        }
    }

    fn flow_cond_true(op: &BinOp, arg1: &Self, arg2: &Self) -> (Self, Self) {
        match op {
            BinOp::Eq => match (arg1, arg2) {
                (Top, _) => (*arg2, *arg2),
                (_, Top) => (*arg1, *arg1),
                _ => (*arg1, *arg2),
            },
            BinOp::Lt => match (arg1, arg2) {
                (Top, Zero) => (Lower, Zero),
                (Top, Lower) => (Lower, Lower),
                (Zero, Top) => (Zero, Greater),
                (Greater, Top) => (Greater, Greater),
                _ => (*arg1, *arg2),
            },
            BinOp::Le => match (arg1, arg2) {
                (Top, Lower) => (Lower, Lower),
                (Greater, Top) => (Greater, Greater),
                _ => (*arg1, *arg2),
            },
            BinOp::Ge => match (arg1, arg2) {
                (Top, Greater) => (Greater, Greater),
                (Lower, Top) => (Lower, Lower),
                _ => (*arg1, *arg2),
            },
            BinOp::Gt => match (arg1, arg2) {
                (Top, Greater) => (Greater, Greater),
                (Top, Zero) => (Greater, Zero),
                (Lower, Top) => (Lower, Lower),
                (Zero, Top) => (Zero, Lower),
                _ => (*arg1, *arg2),
            },
            _ => (*arg1, *arg2),
        }
    }

    fn flow_cond_false(op: &BinOp, arg1: &Self, arg2: &Self) -> (Self, Self) {
        match op {
            BinOp::Lt => match (arg1, arg2) {
                (Top, Greater) => (Greater, Greater),
                (Lower, Top) => (Lower, Lower),
                _ => (*arg1, *arg2),
            },
            BinOp::Le => match (arg1, arg2) {
                (Top, Greater) => (Greater, Greater),
                (Top, Zero) => (Greater, Zero),
                (Lower, Top) => (Lower, Lower),
                (Zero, Top) => (Zero, Lower),
                _ => (*arg1, *arg2),
            },
            BinOp::Ge => match (arg1, arg2) {
                (Top, Zero) => (Lower, Zero),
                (Top, Lower) => (Lower, Lower),
                (Zero, Top) => (Zero, Greater),
                (Greater, Top) => (Greater, Greater),
                _ => (*arg1, *arg2),
            },
            BinOp::Gt => match (arg1, arg2) {
                (Top, Lower) => (Lower, Lower),
                (Greater, Top) => (Greater, Greater),
                _ => (*arg1, *arg2),
            },
            _ => (*arg1, *arg2),
        }
    }
}
