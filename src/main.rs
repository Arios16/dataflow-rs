#![feature(rustc_private)]

extern crate rustc;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_interface;

mod block;

use block::Block;
use rustc::hir::def_id::LOCAL_CRATE;
use rustc::mir::{BasicBlock, BasicBlockData, Mir, StatementKind, TerminatorKind, START_BLOCK};
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

struct Analysis<'tcx> {
    function_mir: &'tcx Mir<'tcx>,
    order: HashMap<BasicBlock, usize>,
    worklist: BinaryHeap<Block<'tcx>>,
}

impl<'tcx> Analysis<'tcx> {
    fn new(function_mir: &'tcx Mir<'tcx>) -> Self {
        let mut order = HashMap::new();
        let mut idx = 0;
        let mut visited = HashSet::new();
        reverse_post_order(
            function_mir.basic_blocks(),
            START_BLOCK,
            &mut order,
            &mut visited,
            &mut idx,
        );
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
        }
    }

    fn print_mir(&self) {
        for (idx, block_data) in self.function_mir.basic_blocks().iter().enumerate() {
            let block_id = BasicBlock::from_usize(idx);
            println!("block_id: {:?}", block_id);
            for stmt in block_data.statements.iter() {
                match stmt.kind {
                    StatementKind::StorageDead(_) | StatementKind::StorageLive(_) => continue, // Not relevant for our purposes
                    _ => println!("\t{:?}", stmt),
                }
            }
            println!("\t{:?}", block_data.terminator());
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
                let (start,end) = (module_name.find('`').unwrap()+1, module_name.rfind('`').unwrap());
                let fn_name = module_name.chars().skip(start).take(end-start).collect::<String>();
                println!("function: {}", fn_name);
                let mir = tcx.optimized_mir(key);
                let analysis = Analysis::new(mir);
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
