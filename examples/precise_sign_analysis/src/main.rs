#![feature(rustc_private)]
extern crate dataflow;

mod lattice;
mod lattice2;

use dataflow::mir::{CastKind, Local, Operand, Place, PlaceBase, Rvalue, Statement, StatementKind};
use dataflow::ty::TyKind;
use std::collections::HashMap;

// f receives a statement `stmt` and the dataflow information associated with that statement `input`.
fn f(stmt: &Statement, input: &HashMap<Local, lattice::PreciseSignAnalysis>) -> Vec<String> {
    let mut r = Vec::new();
    if let StatementKind::Assign(_, ref rvalue) = stmt.kind {
        match &**rvalue {
            Rvalue::Cast(CastKind::Misc, op1, ty) => match ty.sty {
                TyKind::Uint(_) => match op1 {
                    Operand::Copy(place) | Operand::Move(place) => {
                        if let Place::Base(PlaceBase::Local(local)) = place {
                            if input.contains_key(local) {
                                match input[local] {
                                    lattice::PreciseSignAnalysis::Lower => {
                                        r.push(format!("Error at {:?}. Value lower than 0 is being cast as unsigned integer.", 
                                               stmt.source_info.span));
                                    }
                                    lattice::PreciseSignAnalysis::Top
                                    | lattice::PreciseSignAnalysis::LowerEqual => {
                                        r.push(format!("Possible error at {:?}. Value being cast as unsigned integer may be lower than 0.", 
                                                stmt.source_info.span));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
    }
    r
}

fn main() {
    dataflow::run("example.rs", &f);
}
