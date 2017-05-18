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
        match self.0.binary_search(&iv) {
            Ok(_) => {},
            Err(i) => {
                let prev = self.0[i-1];
                let next = if self.0.len() == i { None } else { Some(self.0[i]) };
                if (prev.beg < iv.beg && iv.end <= prev.end) || next.map_or(false, |next| iv.beg == next.beg && iv.end < next.end) {
                    // do nothing
                } else if iv.beg <= prev.end && prev.end < iv.end {
                    // elongate prev forwards and check for possible merges ahead
                    unimplemented!();
                } else if prev.end < iv.beg && next.map_or(true, |next| iv.end < next.beg) {
                    self.0.insert(i, iv);
                } else if prev.end < iv.beg && next.map_or(false, |next| iv.beg <= next.beg && next.beg < iv.end) {
                    // elongate next backwards and check for possible merges ahead
                    unimplemented!();
                } else {
                    unreachable!("Bug! prev: {:?}, next: {:?}, iv: {:?}", prev, next, iv);
                }
            }
        }
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
