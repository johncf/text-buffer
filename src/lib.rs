extern crate xi_rope as rope;

use rope::Rope;

use std::iter::FromIterator;

struct Buffer {
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

/// Half open interval: [beg, end)
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct Interval {
    pub beg: usize,
    pub end: usize,
}

// check whether replacing this with an enum like:
//   { Empty, Single(Interval), Multi(Vec<Interval>) }
// gives any performance gains.
/// Set of ordered disjoint intervals
#[derive(Clone)]
struct IntervalSet(Vec<Interval>);

struct WallView {
    inner: Rope,
    version: u64,
}

struct WallHistEntry {
    added: IntervalSet,
    version: u64,
}

impl IntervalSet {
    pub fn new() -> IntervalSet {
        IntervalSet(Vec::new())
    }

    pub fn add(&mut self, iv: Interval) {
        unimplemented!();
    }
}

impl FromIterator<Interval> for IntervalSet {
    fn from_iter<T>(iter: T) -> Self
        where T: IntoIterator<Item = Interval>
    {
        let mut ret = IntervalSet::new();
        for iv in iter.into_iter() {
            ret.add(iv);
        }
        ret
    }
}

//struct MaskNode {
//    left_nbreaks: usize,
//    left_size: usize,
//}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
