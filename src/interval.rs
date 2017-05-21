use std::{cmp, fmt};
use std::iter::FromIterator;

/// Half open interval: [beg, end)
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Interval {
    pub beg: usize,
    pub end: usize,
}

impl fmt::Debug for Interval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}--{})", self.beg, self.end)
    }
}

impl From<(usize, usize)> for Interval {
    fn from((beg, end): (usize, usize)) -> Interval {
        Interval { beg, end }
    }
}

pub trait IntervalSpace {
    type Info: InfoTy;

    fn compute_info(&self, Interval) -> Self::Info;
}

pub trait InfoTy: Clone {
    fn combine(&self, other: &Self) -> Self;
}

impl InfoTy for () {
    fn combine(&self, _: &()) -> () { () }
}

#[derive(Clone, Debug)]
struct IntervalWrap<T: InfoTy> {
    iv: Interval,
    info: T,
}

pub struct NulSpace;

impl IntervalSpace for NulSpace {
    type Info = ();

    fn compute_info(&self, _: Interval) -> () { () }
}

// check whether replacing this with an enum { Empty, Single(Interval), Multi(Vec<Interval>) }
// gives any performance gains.
/// Set of ordered disjoint intervals
#[derive(Clone)]
pub struct IntervalSet<I: IntervalSpace> {
    space: I,
    inner: Vec<IntervalWrap<I::Info>>, // FIXME optimize; use a tree?
}

#[derive(Clone, Copy)]
pub struct ISpaceShift {
    index: usize,
    offset: usize, // only positive shifts are allowed -- the space can only grow
}

impl<I> fmt::Debug for IntervalSet<I>
where I: IntervalSpace,
      I::Info: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'a, T, U> PartialEq<&'a [T]> for IntervalSet<U>
where T: Into<Interval> + Copy,
      U: IntervalSpace
{
    fn eq(&self, other: &&'a[T]) -> bool {
        self.inner.len() == other.len() && self.inner.iter().zip(*other).all(|(this, other)| this.iv == (*other).into())
    }
}

impl<I: IntervalSpace> IntervalSet<I> {
    pub fn new(space: I) -> IntervalSet<I> {
        IntervalSet {
            space: space,
            inner: Vec::new(),
        }
    }

    pub fn add<T>(&mut self, iv: T) where T: Into<Interval> {
        let iv = iv.into();
        if iv.beg == iv.end { return; }
        assert!(iv.beg < iv.end);

        let beg; let end;
        match self.inner.binary_search_by(|e| e.iv.cmp(&iv)) {
            Ok(_) => {},
            Err(i) => {
                let mut ins_idx = i;
                let prev = if i == 0 { None } else { Some(self.inner[i-1].iv) };
                let next = self.inner.get(i).map(|e| e.iv);
                if prev.map_or(true, |prev| prev.end < iv.beg) &&
                   next.map_or(true, |next| iv.end < next.beg) {
                    beg = iv.beg;
                    end = iv.end;
                } else if prev.map_or(false, |prev| prev.beg < iv.beg && iv.end <= prev.end) ||
                          next.map_or(false, |next| iv.beg == next.beg && iv.end < next.end) {
                    return;
                } else if prev.map_or(false, |prev| iv.beg <= prev.end && prev.end < iv.end) {
                    beg = prev.unwrap().beg;
                    end = self.purge(i-1, iv.end);
                    ins_idx = i-1;
                } else if prev.map_or(true, |prev| prev.end < iv.beg) &&
                          next.map_or(false, |next| iv.beg <= next.beg && next.beg < iv.end) {
                    beg = iv.beg;
                    end = self.purge(i, iv.end);
                } else {
                    unreachable!("Bug! prev: {:?}, next: {:?}, iv: {:?}", prev, next, iv);
                }
                self.inner.insert(ins_idx,
                                  IntervalWrap {
                                      iv: (beg, end).into(),
                                      info: self.space.compute_info((beg, end).into())
                                  });
            }
        }
    }

    fn purge(&mut self, idx: usize, end: usize) -> usize {
        let mut j = self.inner[idx..].binary_search_by(|e| e.iv.cmp(&Interval::from((end, end)))).unwrap_err();
        assert!(j > 0);
        if self.inner.get(idx + j).map_or(false, |e| e.iv.beg == end) {
            j += 1;
        }
        let end = cmp::max(self.inner[idx + j-1].iv.end, end);
        // TODO optimize? compute info only for the "new parts" and combine them with the ones being extended/replaced
        //self.inner[idx].iv.end = end;
        //if j > 1 {
        //    self.inner.drain(idx+1..idx+1 + j-1);
        //}
        self.inner.drain(idx..idx + j);
        end
    }

    pub fn get_space(&self) -> &I {
        &self.space
    }

    /// Note: info for shifted intervals are not recomputed. If shifts are not hollow, then info
    ///       for the added parts alone are computed and then they are combined with the info of
    ///       those intervals they are combined with.
    pub fn update_space(&mut self, new_space: I, shifts: Vec<ISpaceShift>, shifts_are_hollow: bool) {
        let mut cumulative_shift = 0;
        let mut shift_iter = shifts.into_iter();
        let mut shift_peek = shift_iter.next();
        let mut split_add = Vec::new(); // only when not shifts_are_hollow
        for iv_wrap in &mut self.inner {
            if let Some(ISpaceShift { index, offset }) = shift_peek {
                let index_map = index + cumulative_shift;
                if iv_wrap.iv.beg < index {
                    iv_wrap.iv.beg += cumulative_shift;
                    if iv_wrap.iv.end < index {
                        iv_wrap.iv.end += cumulative_shift;
                    } else if iv_wrap.iv.end == index {
                        if shifts_are_hollow {
                            iv_wrap.iv.end += cumulative_shift;
                        } else {
                            iv_wrap.iv.end += cumulative_shift + offset;
                            iv_wrap.info = iv_wrap.info.combine(&new_space.compute_info((index_map, index_map + offset).into()))
                        }
                    } else {
                        let new_end = iv_wrap.iv.end + cumulative_shift + offset;
                        if shifts_are_hollow {
                            iv_wrap.iv.end = new_end;
                            iv_wrap.info = iv_wrap.info.combine(&new_space.compute_info((index_map, index_map + offset).into()))
                        } else {
                            split_add.push((index_map + offset, new_end));
                            iv_wrap.iv.end = index_map;
                            iv_wrap.info = new_space.compute_info(iv_wrap.iv);
                        }
                    }
                } else if iv_wrap.iv.beg == index {
                    iv_wrap.iv.beg += cumulative_shift;
                    iv_wrap.iv.end += cumulative_shift + offset;
                    if shifts_are_hollow {
                        iv_wrap.iv.beg += offset;
                    } else {
                        iv_wrap.info = iv_wrap.info.combine(&new_space.compute_info((index_map, index_map + offset).into()))
                    }
                } else {
                    shift_peek = shift_iter.next();
                    cumulative_shift += offset;
                }
            } else {
                iv_wrap.iv.beg += cumulative_shift;
                iv_wrap.iv.end += cumulative_shift;
            }
        }
        self.space = new_space;
        self.extend(split_add);
    }
}

impl<T, I> Extend<T> for IntervalSet<I>
where T: Into<Interval>,
      I: IntervalSpace
{
    fn extend<U>(&mut self, iter: U)
        where U: IntoIterator<Item = T>
    {
        for e in iter {
            self.add(e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IntervalSet, NulSpace, ISpaceShift};

    #[test]
    fn interval_set_add() {
        let mut ivs = IntervalSet::new(NulSpace);
        ivs.extend(vec![(7, 9), (3, 5)]);
        assert_eq!(ivs, &[(3, 5), (7, 9)]);
        ivs.add((5, 7));
        assert_eq!(ivs, &[(3, 9)]);
        ivs.extend(vec![(7, 8), (11, 13), (15, 16), (18, 19), (10, 17)]);
        assert_eq!(ivs, &[(3, 9), (10, 17), (18, 19)]);
    }

    #[test]
    fn interval_set_space() {
        let mut ivs = IntervalSet::new(NulSpace);
        ivs.extend(vec![(3, 5), (7, 9)]);
        ivs.update_space(NulSpace, vec![ISpaceShift { index: 7, offset: 2 }], true);
        assert_eq!(ivs, &[(3, 5), (9, 11)]);
        ivs.update_space(NulSpace, vec![ISpaceShift { index: 9, offset: 2 }], false);
        assert_eq!(ivs, &[(3, 5), (9, 13)]);
        // TODO more tests (with space info too)
    }
}
