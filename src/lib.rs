extern crate xi_rope as rope;

mod interval;

use interval::{Interval, IntervalSet, IntervalSpace, InfoTy};

use rope::Rope;

pub struct Buffer {
    wall: Wall,
    mask: Mask,
}

struct Wall {
    inner: Rope,
    hist: Vec<WallHistEntry>,
    version: u64,
}

struct Mask {
    inner: IntervalSet<MaskSpace>,
}

struct MaskSpace {
    wall: WallView,
}

impl IntervalSpace for MaskSpace {
    type Info = usize;

    fn compute_info(&self, iv: Interval) -> usize {
        0 // TODO
    }
}

struct WallView {
    inner: Rope,
    version: u64,
}

struct WallHistEntry {
    added: IntervalSet<interval::NulSpace>,
    version: u64,
}

//struct MaskNode {
//    left_nbreaks: usize,
//    left_size: usize,
//}
