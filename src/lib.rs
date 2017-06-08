extern crate xi_rope as rope;

pub mod interval;

use interval::Interval;

use rope::Rope;

pub struct Buffer {
    wall: Wall,
    mask: Mask,
}

struct Wall {
    inner: Rope,
    hist: WallHistory,
    version: u32,
}

struct WallHistory {
    entries: Vec<WallAdd>, // sorted
}

struct WallAdd {
    version: u32, // sorted on this
    index: usize,
    size: usize,
}

struct Mask {
    inner: StrSpanSet,
    hist: UndoHistory,
}

type StrSpanSet = !; // TODO a unitree with StrSpan as leaves

struct StrSpan {
    offset: usize,
    size: usize,
    lines: usize, // number of lines in this span
}

struct UndoHistory { // linear undo history
    inner: Vec<UndoEntry>,
}

struct UndoEntry {
    flips: Vec<Interval>, // intervals that change visibility
    wall_version: u32, // version of the wall intervals are based on
}
