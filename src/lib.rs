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
#[derive(Clone, Copy, Debug, PartialOrd, Eq, Ord)]
struct Interval {
    pub beg: usize,
    pub end: usize,
}

impl<T> PartialEq<T> for Interval where T: Copy + Into<Interval> {
    fn eq(&self, other: &T) -> bool {
        let other = (*other).into();
        self.beg == other.beg && self.end == other.end
    }
}

impl From<(usize, usize)> for Interval {
    fn from((beg, end): (usize, usize)) -> Interval {
        Interval { beg, end }
    }
}

// check whether replacing this with an enum like:
//   { Empty, Single(Interval), Multi(Vec<Interval>) }
// gives any performance gains.
/// Set of ordered disjoint intervals
#[derive(Clone, Debug)]
struct IntervalSet(Vec<Interval>);

impl<'a, T> PartialEq<&'a [T]> for IntervalSet
where T: Copy + Into<Interval>
{
    fn eq(&self, other: &&'a[T]) -> bool {
        self.0.len() == other.len() && self.0.iter().zip(*other).all(|(this, other)| this == other)
    }
}

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

    pub fn add<T>(&mut self, iv: T) where T: Into<Interval> {
        let iv = iv.into();
        if iv.beg == iv.end { return; }
        match self.0.binary_search(&iv) {
            Ok(_) => {},
            Err(i) => {
                let prev = if i == 0 { None } else { Some(self.0[i-1]) };
                let next = if self.0.len() == i { None } else { Some(self.0[i]) };
                if prev.map_or(false, |prev| prev.beg < iv.beg && iv.end <= prev.end) ||
                   next.map_or(false, |next| iv.beg == next.beg && iv.end < next.end) {
                    // do nothing
                } else if prev.map_or(false, |prev| iv.beg <= prev.end && prev.end < iv.end) {
                    // elongate prev forwards and check for possible merges ahead
                    let j = self.0[i-1..].binary_search(&Interval::from((iv.end, iv.end))).unwrap_err();
                    assert!(j > 0);
                    self.0[i-1].end = std::cmp::max(self.0[i-1 + j-1].end, iv.end);
                    if j > 1 {
                        self.0.drain(i..i + j-1);
                    }
                } else if prev.map_or(true, |prev| prev.end < iv.beg) &&
                          next.map_or(true, |next| iv.end < next.beg) {
                    self.0.insert(i, iv);
                } else if prev.map_or(true, |prev| prev.end < iv.beg) &&
                          next.map_or(false, |next| iv.beg <= next.beg && next.beg < iv.end) {
                    // elongate next backwards and check for possible merges ahead
                    let j = self.0[i..].binary_search(&Interval::from((iv.end, iv.end))).unwrap_err();
                    assert!(j > 0);
                    self.0[i].beg = iv.beg;
                    self.0[i].end = std::cmp::max(self.0[i + j-1].end, iv.end);
                    if j > 1 {
                        self.0.drain(i+1..i+1 + j-1);
                    }
                } else {
                    unreachable!("Bug! prev: {:?}, next: {:?}, iv: {:?}", prev, next, iv);
                }
            }
        }
    }
}

impl<T> FromIterator<T> for IntervalSet where T: Into<Interval> {
    fn from_iter<U>(iter: U) -> Self
        where U: IntoIterator<Item = T>
    {
        let mut ret = IntervalSet::new();
        for iv in iter {
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
    use super::IntervalSet;

    #[test]
    fn interval_set_add() {
        let ivs: IntervalSet = vec![(8, 9), (5, 7)].into_iter().collect();
        assert_eq!(ivs, &[(5, 7), (8, 9)]);
        let ivs: IntervalSet = vec![(3, 5), (5, 7)].into_iter().collect();
        assert_eq!(ivs, &[(3, 7)]);
        let ivs: IntervalSet = vec![(3, 5), (6, 7), (5, 6)].into_iter().collect();
        assert_eq!(ivs, &[(3, 7)]);
    }
}
