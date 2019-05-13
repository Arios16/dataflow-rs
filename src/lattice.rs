use rustc::mir;
use rustc::mir::interpret::ConstValue;
use rustc::mir::{BinOp, Operand, Place, PlaceBase, Rvalue, UnOp};
use rustc::mir::{Local, LocalDecl};
use rustc::ty::TyKind;
use rustc_data_structures::indexed_vec::IndexVec;
use std::collections::HashMap;

pub trait SimpleLattice: PartialEq + Eq + Copy + std::fmt::Debug {
    fn applies(ty: &TyKind) -> bool;
    fn bot() -> Self;
    fn top() -> Self;
    fn join(op1: &Self, op2: &Self) -> Self;
    fn alpha(a: mir::interpret::ConstValue) -> Self;
    fn flow_binop(op: &BinOp, arg1: &Self, arg2: &Self) -> Self;
    fn flow_unop(op: &UnOp, arg: &Self) -> Self;
    fn flow_cond_true(op: &BinOp, arg1: &Self, arg2: &Self) -> (Self, Self);
    fn flow_cond_false(op: &BinOp, arg1: &Self, arg2: &Self) -> (Self, Self);
}

pub trait Lattice: PartialEq + Eq + Sized + Clone + std::fmt::Debug {
    fn bot(decls: &IndexVec<Local, LocalDecl>) -> Self;
    fn top(decls: &IndexVec<Local, LocalDecl>) -> Self;
    fn join(op1: &Self, op2: &Self) -> Self;
    fn flow_assign(&self, local: Local, rvalue: &Box<Rvalue>, equiv: &mut HashMap<Local, Vec<Local>>) -> Self;
    fn flow_branch(&self, rvalue: &Box<Rvalue>, equiv: &mut HashMap<Local, Vec<Local>>) -> (Self, Self);
    fn flow_function_call(&self, func: &Operand, args: &Vec<Operand>, destination: &Place) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignAnalysisSimpleLattice {
    Top,
    Bottom,
    Lower,
    Zero,
    Greater,
}

use SignAnalysisSimpleLattice::*;
impl SimpleLattice for SignAnalysisSimpleLattice {
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

impl<SL: SimpleLattice> Lattice for HashMap<Local, SL> {
    fn bot(decls: &IndexVec<Local, LocalDecl>) -> Self {
        let mut r = HashMap::new();
        for (local, decl) in decls.iter_enumerated() {
            if SL::applies(&decl.ty.sty) {
                r.insert(local, SL::bot());
            }
        }
        r
    }

    fn top(decls: &IndexVec<Local, LocalDecl>) -> Self {
        let mut r = HashMap::new();
        for (local, decl) in decls.iter_enumerated() {
            if SL::applies(&decl.ty.sty) {
                r.insert(local, SL::top());
            }
        }
        r
    }

    fn join(op1: &Self, op2: &Self) -> Self {
        let mut newlattice = HashMap::new();
        for key in op1.keys() {
            newlattice.insert(*key, SL::join(&op1[key], &op2[key]));
        }
        newlattice
    }

    fn flow_assign(&self, local: Local, rvalue: &Box<Rvalue>, equiv: &mut HashMap<Local, Vec<Local>>) -> Self {
        if !self.contains_key(&local) {
            return self.clone();
        }
        let get_val = |op: &Operand| match op {
            Operand::Copy(place) | Operand::Move(place) => match place {
                Place::Base(place_base) => match place_base {
                    PlaceBase::Local(local2) => self[local2].clone(),
                    _ => SL::top(),
                },
                _ => SL::top(),
            },
            Operand::Constant(constant) => SL::alpha(constant.literal.val),
        };
        let get_local = |op: &Operand| match op {
            Operand::Copy(place) | Operand::Move(place) => match place {
                Place::Base(place_base) => match place_base {
                    PlaceBase::Local(local) => Some(local.clone()),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        };

        let mut newlattice = self.clone();
        let val = match &**rvalue {
            Rvalue::Use(op) => {
                if let Some(local2) = get_local(op){
                    equiv.entry(local).or_insert_with(|| Vec::new()).push(local2);
                    equiv.entry(local2).or_insert_with(|| Vec::new()).push(local);
                }
                get_val(op)
            },
            Rvalue::BinaryOp(op, op1, op2) | Rvalue::CheckedBinaryOp(op, op1, op2) => {
                let op1 = get_val(op1);
                let op2 = get_val(op2);
                SL::flow_binop(op, &op1, &op2)
            }
            Rvalue::UnaryOp(op, op1) => {
                let op1 = get_val(op1);
                SL::flow_unop(op, &op1)
            }
            _ => SL::top(),
        };
        *(newlattice.get_mut(&local).unwrap()) = val;
        return newlattice;
    }

    fn flow_branch(&self, rvalue: &Box<Rvalue>, equiv: &mut HashMap<Local, Vec<Local>>) -> (Self, Self) {
        let get_val = |op: &Operand| match op {
            Operand::Copy(place) | Operand::Move(place) => match place {
                Place::Base(place_base) => match place_base {
                    PlaceBase::Local(local2) => self[local2].clone(),
                    _ => SL::top(),
                },
                _ => SL::top(),
            },
            Operand::Constant(constant) => SL::alpha(constant.literal.val),
        };
        let get_local = |op: &Operand| match op {
            Operand::Copy(place) | Operand::Move(place) => match place {
                Place::Base(place_base) => match place_base {
                    PlaceBase::Local(local) => Some(local.clone()),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        };

        match &**rvalue {
            Rvalue::BinaryOp(op, op1, op2) | Rvalue::CheckedBinaryOp(op, op1, op2) => {
                let local1 = get_local(op1);
                let local2 = get_local(op2);

                if local1.is_some() && self.contains_key(&local1.unwrap())
                    || local2.is_some() && self.contains_key(&local2.unwrap())
                {
                    let op1 = get_val(op1);
                    let op2 = get_val(op2);

                    let (l1, l2) = SL::flow_cond_false(op, &op1, &op2);
                    let mut lattice1 = self.clone();
                    if let Some(local) = local1 {
                        *(lattice1.get_mut(&local).unwrap()) = l1;
                        if let Some(equivs) = equiv.get(&local){
                            for local3 in equivs.iter(){
                                *(lattice1.get_mut(&local3).unwrap()) = l1;
                            }
                        }
                    }
                    if let Some(local) = local2 {
                        *(lattice1.get_mut(&local).unwrap()) = l2;
                        if let Some(equivs) = equiv.get(&local){
                            for local3 in equivs.iter(){
                                *(lattice1.get_mut(&local3).unwrap()) = l2;
                            }
                        }
                    }

                    let (l1, l2) = SL::flow_cond_true(op, &op1, &op2);
                    let mut lattice2 = self.clone();
                    if let Some(local) = local1 {
                        *(lattice2.get_mut(&local).unwrap()) = l1;
                        if let Some(equivs) = equiv.get(&local){
                            for local3 in equivs.iter(){
                                *(lattice2.get_mut(&local3).unwrap()) = l1;
                            }
                        }
                    }
                    if let Some(local) = local2 {
                        *(lattice2.get_mut(&local).unwrap()) = l2;
                        if let Some(equivs) = equiv.get(&local){
                            for local3 in equivs.iter(){
                                *(lattice2.get_mut(&local3).unwrap()) = l2;
                            }
                        }
                    }
                    (lattice1, lattice2)
                } else {
                    (self.clone(), self.clone())
                }

            }
            _ => (self.clone(), self.clone()),
        }
    }

    fn flow_function_call(&self, _func: &Operand, _args: &Vec<Operand>, destination: &Place) -> Self {
        let mut r = self.clone();
        match destination {
            Place::Base(place_base) => match place_base {
                PlaceBase::Local(local) => {
                    if let Some(p) = r.get_mut(&local) {
                        *p = SL::top();
                    }
                }
                _ => {}
            },
            _ => {}
        }
        r
    }
}
