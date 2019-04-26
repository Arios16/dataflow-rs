use rustc::mir::{BasicBlock, BasicBlockData};

pub struct Block<'tcx> {
    pub id: BasicBlock,
    pub data: &'tcx BasicBlockData<'tcx>,
    pub idx: usize,
}

impl<'tcx> PartialEq for Block<'tcx> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'tcx> Eq for Block<'tcx> {}

impl<'tcx> Ord for Block<'tcx> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.id.cmp(&self.id) // Note that the order is reversed (so we can use std::collections::BinaryHeap as a min-heap)
    }
}

impl<'tcx> PartialOrd for Block<'tcx> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'tcx> Block<'tcx> {
    pub fn new(id: BasicBlock, data: &'tcx BasicBlockData<'tcx>, idx: usize) -> Self {
        Self { id, data, idx }
    }
}
