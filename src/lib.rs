extern crate xi_rope as rope;

pub mod interval;

use interval::{Interval, IntervalSet, IntervalSpace};

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

    fn compute_info(&self, _iv: Interval) -> usize {
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
