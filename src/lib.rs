extern crate xi_rope as rope;

mod interval;

use interval::{Interval, IntervalSet};

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
    inner: Vec<MaskPiece>, // FIXME optimize; use a tree
    wall: WallView,
}

struct MaskPiece {
    range: Interval,
    nbreaks: usize,
}

struct WallView {
    inner: Rope,
    version: u64,
}

struct WallHistEntry {
    added: IntervalSet,
    version: u64,
}

//struct MaskNode {
//    left_nbreaks: usize,
//    left_size: usize,
//}
