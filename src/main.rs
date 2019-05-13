#![feature(rustc_private)]

extern crate rustc;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_interface;

mod block;
mod lattice;

use block::Block;
use rustc::hir::def_id::LOCAL_CRATE;
use rustc::mir::{
    BasicBlock, BasicBlockData, Local, Mir, Operand, Place, PlaceBase, StatementKind,
    TerminatorKind, START_BLOCK,
};
use rustc::ty::TyKind;
use rustc_data_structures::indexed_vec::IndexVec;
use rustc_interface::interface;
use std::collections::{BinaryHeap, HashMap, HashSet};

fn reverse_post_order(
    blocks: &IndexVec<BasicBlock, BasicBlockData>,
    block: BasicBlock,
    order: &mut HashMap<BasicBlock, usize>,
    visited: &mut HashSet<BasicBlock>,
    idx: &mut usize,
) {
    if visited.contains(&block) {
        return;
    } else {
        visited.insert(block);
    }
    let data = &blocks[block];
    for &suc in data.terminator().successors() {
        reverse_post_order(blocks, suc, order, visited, idx);
    }
    order.insert(block, *idx);
    *idx += 1;
}

struct Analysis<'tcx, L: lattice::Lattice> {
    function_mir: &'tcx Mir<'tcx>,
    input: IndexVec<BasicBlock, L>,
    order: HashMap<BasicBlock, usize>,
    worklist: BinaryHeap<Block<'tcx>>,
}

impl<'tcx, L: lattice::Lattice> Analysis<'tcx, L> {
    fn new(function_mir: &'tcx Mir<'tcx>) -> Self {
        let mut order = HashMap::new();
        let mut idx = 0;
        let mut visited = HashSet::new();
        let mut input = IndexVec::new();
        reverse_post_order(
            function_mir.basic_blocks(),
            START_BLOCK,
            &mut order,
            &mut visited,
            &mut idx,
        );

        for _ in 0..function_mir.basic_blocks().len() {
            input.push(L::bot(&function_mir.local_decls));
        }
        input[START_BLOCK] = L::top(&function_mir.local_decls);
        let mut worklist = BinaryHeap::new();
        let first_block = Block::new(
            START_BLOCK,
            &function_mir.basic_blocks()[START_BLOCK],
            order[&START_BLOCK],
        );
        worklist.push(first_block);
        Self {
            function_mir,
            order,
            worklist,
            input,
        }
    }

    fn run(&mut self) {
        while let Some(block) = self.worklist.pop() {
            let mut lattice = self.input[block.id].clone();
            let mut lattice2 = None;
            let mut if_local_bool = None;
            let mut equivs = HashMap::new();

            // To be able to propagate conditional information
            match block.data.terminator().kind {
                TerminatorKind::SwitchInt {
                    ref discr,
                    ref switch_ty,
                    values: _,
                    targets: _,
                } => {
                    if let TyKind::Bool = switch_ty.sty {
                        match discr {
                            Operand::Copy(place) | Operand::Move(place) => match place {
                                Place::Base(PlaceBase::Local(local)) => {
                                    if_local_bool = Some(local);
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                }
                _ => {}
            }

            // Process statements in this block
            let mut finished = false;
            for stmt in block.data.statements.iter() {
                match stmt.kind {
                    StatementKind::Assign(ref place, ref rvalue) => match place {
                        Place::Base(place_base) => match place_base {
                            PlaceBase::Local(local) => {
                                if if_local_bool == Some(local) {
                                    let r = lattice.flow_branch(rvalue, &mut equivs);
                                    lattice = r.0;
                                    lattice2 = Some(r.1);
                                    finished = true;
                                } else {
                                    assert!(!finished);
                                    lattice = lattice.flow_assign(*local, rvalue, &mut equivs);
                                    println!("stmt: {:?}, {:?}", stmt, lattice);
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    _ => {} // We only really care about assignments
                }
            }

            // Process function call if it exists
            match block.data.terminator().kind {
                TerminatorKind::Call {
                    ref func,
                    ref args,
                    ref destination,
                    cleanup: _,
                    from_hir_call: _,
                } => {
                    if let Some((place, _)) = destination {
                        lattice = lattice.flow_function_call(func, args, place);
                    }
                }
                _ => {}
            }

            // Propagate information to block successors and add to worklist
            let successors = block
                .data
                .terminator()
                .successors()
                .cloned()
                .collect::<Vec<BasicBlock>>();
            if if_local_bool.is_some(){
                self.input[successors[0]] = L::join(&lattice, &self.input[successors[0]]);
                self.input[successors[1]] = L::join(&lattice2.unwrap(), &self.input[successors[1]]);
                let b1 = Block::new(
                    successors[0],
                    &self.function_mir.basic_blocks()[successors[0]],
                    self.order[&successors[0]],
                );
                let b2 = Block::new(
                    successors[1],
                    &self.function_mir.basic_blocks()[successors[1]],
                    self.order[&successors[1]],
                );
                self.worklist.push(b1);
                self.worklist.push(b2);
            } else {
                for suc in successors.into_iter(){
                    self.input[suc] = L::join(&lattice, &self.input[suc]);
                    let b = Block::new(
                        suc,
                        &self.function_mir.basic_blocks()[suc],
                        self.order[&suc]
                    );
                    self.worklist.push(b);
                }
            }
        }
    }

    fn print_mir(&self) {
        for (idx, block_data) in self.function_mir.basic_blocks().iter().enumerate() {
            let block_id = BasicBlock::from_usize(idx);
            println!("\t{} - block_id: {:?}", idx, block_id);
            for stmt in block_data.statements.iter() {
                match stmt.kind {
                    // StatementKind::StorageDead(_) | StatementKind::StorageLive(_) => continue, // Not relevant for our purposes
                    // _ => println!("\t\t{:?} {:?}", stmt, stmt.kind),
                    StatementKind::Assign(ref place, ref rvalue) => {
                        println!("\t\t place: {:?}, rvalue: {:?}", place, rvalue);
                    }
                    _ => {} // We only really care about assignments
                }
            }
            println!("\t\t{:?}", block_data.terminator());
            match block_data.terminator().kind {
                TerminatorKind::Call {
                    func: ref f,
                    args: _,
                    destination: ref d,
                    cleanup: _,
                    from_hir_call: _,
                } => {
                    println!("\t\tf: {:?}", f);
                    println!("\t\td: {:?}", d);
                }
                TerminatorKind::Goto { target: block } => {
                    println!("\t\tblock: {:?}", block);
                }
                TerminatorKind::SwitchInt {
                    ref discr,
                    switch_ty: ref ty,
                    ref values,
                    ref targets,
                } => {
                    println!(
                        "\t\tdiscr: {:?}, switch_ty: {:?}, values: {:?}, targets: {:?}",
                        discr, ty, values, targets
                    );
                }
                _ => {}
            }
        }
    }
}

struct CompilerCallback;

impl rustc_driver::Callbacks for CompilerCallback {
    fn after_analysis(&mut self, compiler: &interface::Compiler) -> bool {
        compiler.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            // let (main_id, _) = tcx.entry_fn(LOCAL_CRATE).unwrap();
            let set = tcx.mir_keys(LOCAL_CRATE);
            for &key in set.iter() {
                let module_name = key.describe_as_module(tcx);
                let (start, end) = (
                    module_name.find('`').unwrap() + 1,
                    module_name.rfind('`').unwrap(),
                );
                let fn_name = module_name
                    .chars()
                    .skip(start)
                    .take(end - start)
                    .collect::<String>();
                if fn_name != "test_fn" {
                    continue;
                }
                println!("function: {}", fn_name);
                println!("function: {}", module_name);
                let mir = tcx.optimized_mir(key);
                // for (local, decl) in mir.local_decls.iter_enumerated() {
                //     println!("{:?} {:?}", local, decl.ty);
                //     match decl.ty.sty {
                //         TyKind::Int(a) => {
                //             println!("{:?}", a);
                //         }
                //         _ => {}
                //     }
                // }
                let mut analysis =
                    Analysis::<HashMap<Local, lattice::SignAnalysis>>::new(mir);
                analysis.run();
                println!("{:?}", analysis.input);
                analysis.print_mir();
                println!();
            }
        });

        false // no need to continue compilation (we only cared enough to extract the MIR and do the analysis)
    }
}

fn main() {
    let exe = std::env::current_exe().unwrap();
    let exe = exe.to_str().unwrap();
    let sysroot = match std::env::var_os("RUST_SYSROOT") {
        Some(val) => val.into_string().unwrap(),
        None => {
            println!(
                "Cannot find Rust Sysroot. Please define the `RUST_SYSROOT` environment variable"
            );
            return;
        }
    };
    let args = vec![exe, "example.rs", "--sysroot", &sysroot, "-O"];
    let args = args.into_iter().map(|x| x.to_owned()).collect::<Vec<_>>();
    rustc_driver::run_compiler(&args[..], &mut CompilerCallback, None, None).unwrap();
}
