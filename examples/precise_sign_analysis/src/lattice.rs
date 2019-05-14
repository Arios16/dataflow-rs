use dataflow::mir::interpret::ConstValue;
use dataflow::mir::{BinOp, UnOp};
use dataflow::ty::TyKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreciseSignAnalysis {
    Top,
    Bottom,
    Lower,
    LowerEqual,
    Zero,
    Greater,
    GreaterEqual,
}

use PreciseSignAnalysis::*;

// The implementation for the flow functions is not very pretty, but I cant think of an elegant way to do it
impl dataflow::lattice::SimpleLattice for PreciseSignAnalysis {
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
            (Bottom, a) | (a, Bottom) => a,
            (Greater, GreaterEqual)
            | (GreaterEqual, Greater)
            | (Zero, GreaterEqual)
            | (GreaterEqual, Zero)
            | (Zero, Greater)
            | (Greater, Zero) => GreaterEqual,
            (Lower, LowerEqual)
            | (LowerEqual, Lower)
            | (Zero, LowerEqual)
            | (LowerEqual, Zero)
            | (Zero, Lower)
            | (Lower, Zero) => LowerEqual,
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
                (Greater, Zero)
                | (Zero, Greater)
                | (Greater, Greater)
                | (Greater, GreaterEqual)
                | (GreaterEqual, Greater) => Greater,
                (Lower, Zero)
                | (Zero, Lower)
                | (Lower, Lower)
                | (Lower, LowerEqual)
                | (LowerEqual, Lower) => Lower,
                (GreaterEqual, GreaterEqual) | (Zero, GreaterEqual) | (GreaterEqual, Zero) => {
                    GreaterEqual
                }
                (LowerEqual, LowerEqual) | (Zero, LowerEqual) | (LowerEqual, Zero) => LowerEqual,
                (Zero, Zero) => Zero,
                _ => Top,
            },
            BinOp::Sub => match (arg1, arg2) {
                (Greater, Zero)
                | (Zero, Lower)
                | (Greater, Lower)
                | (GreaterEqual, Lower)
                | (Greater, LowerEqual) => Greater,
                (Zero, Greater)
                | (Lower, Greater)
                | (Lower, Zero)
                | (LowerEqual, Greater)
                | (Lower, GreaterEqual) => Lower,
                (Zero, GreaterEqual) | (LowerEqual, GreaterEqual) | (LowerEqual, Zero) => {
                    LowerEqual
                }
                (Zero, LowerEqual) | (GreaterEqual, LowerEqual) | (GreaterEqual, Zero) => {
                    GreaterEqual
                }
                (Zero, Zero) => Zero,
                _ => Top,
            },
            BinOp::Mul => match (arg1, arg2) {
                (Zero, _) | (_, Zero) => Zero,
                (Lower, Lower) | (Greater, Greater) => Greater,
                (Greater, Lower) | (Lower, Greater) => Lower,
                (GreaterEqual, GreaterEqual)
                | (GreaterEqual, Greater)
                | (Greater, GreaterEqual) => GreaterEqual,
                (LowerEqual, LowerEqual) | (LowerEqual, Lower) | (Lower, LowerEqual) => {
                    GreaterEqual
                }
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
                GreaterEqual => LowerEqual,
                LowerEqual => GreaterEqual,
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
                (GreaterEqual, Greater) | (Greater, GreaterEqual) => (Greater, Greater),
                (LowerEqual, Lower) | (Lower, LowerEqual) => (Lower, Lower),
                (GreaterEqual, Zero) | (Zero, GreaterEqual) => (Zero, Zero),
                (LowerEqual, Zero) | (Zero, LowerEqual) => (Zero, Zero),
                _ => (*arg1, *arg2),
            },
            BinOp::Lt => match (arg1, arg2) {
                (Top, Zero) | (LowerEqual, Zero) => (Lower, Zero),
                (Top, Lower) | (LowerEqual, Lower) => (Lower, Lower),
                (Zero, Top) | (Zero, GreaterEqual) => (Zero, Greater),
                (Greater, Top) | (Greater, GreaterEqual) => (Greater, Greater),
                (Top, LowerEqual) => (LowerEqual, LowerEqual),
                _ => (*arg1, *arg2),
            },
            BinOp::Le => match (arg1, arg2) {
                (Top, Lower) | (LowerEqual, Lower) => (Lower, Lower),
                (Top, LowerEqual) => (LowerEqual, LowerEqual),
                (Greater, Top) | (Greater, GreaterEqual) => (Greater, Greater),
                (Top, Zero) => (LowerEqual, Zero),
                (Zero, Top) => (Zero, GreaterEqual),
                (GreaterEqual, Top) => (GreaterEqual, GreaterEqual),
                _ => (*arg1, *arg2),
            },
            BinOp::Ge => match (arg1, arg2) {
                (Top, Greater) | (GreaterEqual, Greater) => (Greater, Greater),
                (Top, GreaterEqual) => (GreaterEqual, GreaterEqual),
                (Lower, Top) | (Lower, LowerEqual) => (Lower, Lower),
                (Top, Zero) => (GreaterEqual, Zero),
                (Zero, Top) => (Zero, LowerEqual),
                (LowerEqual, Top) => (LowerEqual, LowerEqual),
                _ => (*arg1, *arg2),
            },
            BinOp::Gt => match (arg1, arg2) {
                (Top, Zero) | (GreaterEqual, Zero) => (Greater, Zero),
                (Top, Greater) | (GreaterEqual, Greater) => (Greater, Greater),
                (Zero, Top) | (Zero, LowerEqual) => (Zero, Lower),
                (Lower, Top) | (Lower, LowerEqual) => (Lower, Lower),
                (Top, GreaterEqual) => (GreaterEqual, GreaterEqual),
                _ => (*arg1, *arg2),
            },
            _ => (*arg1, *arg2),
        }
    }

    fn flow_cond_false(op: &BinOp, arg1: &Self, arg2: &Self) -> (Self, Self) {
        match op {
            BinOp::Lt => match (arg1, arg2) {
                (Top, Greater) | (GreaterEqual, Greater) => (Greater, Greater),
                (Top, GreaterEqual) => (GreaterEqual, GreaterEqual),
                (Lower, Top) | (Lower, LowerEqual) => (Lower, Lower),
                (Top, Zero) => (GreaterEqual, Zero),
                (Zero, Top) => (Zero, LowerEqual),
                (LowerEqual, Top) => (LowerEqual, LowerEqual),
                _ => (*arg1, *arg2),
            },
            BinOp::Le => match (arg1, arg2) {
                (Top, Zero) | (GreaterEqual, Zero) => (Greater, Zero),
                (Top, Greater) | (GreaterEqual, Greater) => (Greater, Greater),
                (Zero, Top) | (Zero, LowerEqual) => (Zero, Lower),
                (Lower, Top) | (Lower, LowerEqual) => (Lower, Lower),
                (Top, GreaterEqual) => (GreaterEqual, GreaterEqual),
                _ => (*arg1, *arg2),
            },
            BinOp::Ge => match (arg1, arg2) {
                (Top, Zero) | (LowerEqual, Zero) => (Lower, Zero),
                (Top, Lower) | (LowerEqual, Lower) => (Lower, Lower),
                (Zero, Top) | (Zero, GreaterEqual) => (Zero, Greater),
                (Greater, Top) | (Greater, GreaterEqual) => (Greater, Greater),
                (Top, LowerEqual) => (LowerEqual, LowerEqual),
                _ => (*arg1, *arg2),
            },
            BinOp::Gt => match (arg1, arg2) {
                (Top, Lower) | (LowerEqual, Lower) => (Lower, Lower),
                (Top, LowerEqual) => (LowerEqual, LowerEqual),
                (Greater, Top) | (Greater, GreaterEqual) => (Greater, Greater),
                (Top, Zero) => (LowerEqual, Zero),
                (Zero, Top) => (Zero, GreaterEqual),
                (GreaterEqual, Top) => (GreaterEqual, GreaterEqual),
                _ => (*arg1, *arg2),
            },
            _ => (*arg1, *arg2),
        }
    }
}
