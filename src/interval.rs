use std::{cmp, fmt};
use std::iter::FromIterator;

/// Half open interval: [beg, end)
#[derive(Clone, Copy, PartialOrd, Eq, Ord)]
pub struct Interval {
    pub beg: usize,
    pub end: usize,
}

impl fmt::Debug for Interval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}--{})", self.beg, self.end)
    }
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

// check whether replacing this with an enum { Empty, Single(Interval), Multi(Vec<Interval>) }
// gives any performance gains.
/// Set of ordered disjoint intervals
#[derive(Clone, Debug)]
pub struct IntervalSet(Vec<Interval>);

impl<'a, T> PartialEq<&'a [T]> for IntervalSet
where T: Copy + Into<Interval>
{
    fn eq(&self, other: &&'a[T]) -> bool {
        self.0.len() == other.len() && self.0.iter().zip(*other).all(|(this, other)| this == other)
    }
}

impl IntervalSet {
    pub fn new() -> IntervalSet {
        IntervalSet(Vec::new())
    }

    pub fn add<T>(&mut self, iv: T) where T: Into<Interval> {
        let iv = iv.into();
        if iv.beg == iv.end { return; }
        assert!(iv.beg < iv.end);

        match self.0.binary_search(&iv) {
            Ok(_) => {},
            Err(i) => {
                let prev = if i == 0 { None } else { Some(self.0[i-1]) };
                let next = self.0.get(i).map(|iv| *iv);
                if prev.map_or(true, |prev| prev.end < iv.beg) &&
                   next.map_or(true, |next| iv.end < next.beg) {
                    self.0.insert(i, iv);
                } else if prev.map_or(false, |prev| prev.beg < iv.beg && iv.end <= prev.end) ||
                          next.map_or(false, |next| iv.beg == next.beg && iv.end < next.end) {
                    // do nothing
                } else if prev.map_or(false, |prev| iv.beg <= prev.end && prev.end < iv.end) {
                    self.elongate(i-1, iv.end);
                } else if prev.map_or(true, |prev| prev.end < iv.beg) &&
                          next.map_or(false, |next| iv.beg <= next.beg && next.beg < iv.end) {
                    self.0[i].beg = iv.beg;
                    self.elongate(i, iv.end);
                } else {
                    unreachable!("Bug! prev: {:?}, next: {:?}, iv: {:?}", prev, next, iv);
                }
            }
        }
    }

    fn elongate(&mut self, idx: usize, end: usize) {
        let mut j = self.0[idx..].binary_search(&Interval::from((end, end))).unwrap_err();
        assert!(j > 0);
        if self.0.get(idx + j).map_or(false, |iv| iv.beg == end) {
            j += 1;
        }
        self.0[idx].end = cmp::max(self.0[idx + j-1].end, end);
        if j > 1 {
            self.0.drain(idx+1..idx+1 + j-1);
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

#[cfg(test)]
mod tests {
    use super::IntervalSet;

    #[test]
    fn interval_set_add() {
        let mut ivs: IntervalSet = vec![(7, 9), (3, 5)].into_iter().collect();
        assert_eq!(ivs, &[(3, 5), (7, 9)]);
        ivs.add((5, 7));
        assert_eq!(ivs, &[(3, 9)]);
        let ivs: IntervalSet = vec![(2, 3), (5, 7), (9, 10), (12, 13), (4, 11)].into_iter().collect();
        assert_eq!(ivs, &[(2, 3), (4, 11), (12, 13)]);
    }
}
