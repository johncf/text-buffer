use std::{cmp, fmt};

/// Half open interval: [beg, end)
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Interval {
    pub beg: usize,
    pub end: usize,
}

// check whether replacing this with an enum { Empty, Single(Interval), Multi(Vec<Interval>) }
// gives any performance gains.
/// A set of non-overlapping intervals of the form [x, y) where x < y.
#[derive(Clone)]
pub struct IntervalSet<I: IntervalSpace> {
    space: I,
    inner: Vec<IntervalWrap<I::Info>>, // FIXME optimize; use a tree?
}

/// A growth in space of intervals.
///
/// If the interval space is grown by a single `ISpaceAdd`, every interval to the right of, and
/// including, `index` gets shifted right by `size` amount. It is assumed that everything outside
/// of the added space remains the same, and therefore all metadata about existing intervals remain
/// unchanged.
#[derive(Clone, Copy)]
pub struct ISpaceAdd {
    /// The index at which space is added
    pub index: usize,
    /// The size of the added space
    pub size: usize,
}

pub trait IntervalSpace {
    type Info: InfoTy;

    fn compute_info(&self, Interval) -> Self::Info;
}

pub trait InfoTy: Clone {
    fn combine(&self, other: &Self) -> Self;
}

impl Interval {
    /// Note: Make sure `self.iv.beg >= by`
    fn shift_left(&mut self, by: usize) {
        self.beg -= by;
        self.end -= by;
    }

    /// Note: Make sure `self.iv.end <= usize::MAX - by`
    fn shift_right(&mut self, by: usize) {
        self.beg += by;
        self.end += by;
    }

    /// Panics if `from` is at either boundary of the interval
    fn split_off(&mut self, from: usize) -> Interval {
        assert!(self.beg < from && from < self.end);
        let ret = Interval { beg: from, end: self.end };
        self.end = from;
        ret
    }
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

impl From<ISpaceAdd> for Interval {
    fn from(add: ISpaceAdd) -> Interval {
        Interval { beg: add.index, end: add.index + add.size }
    }
}

impl InfoTy for () {
    fn combine(&self, _: &()) -> () { () }
}

impl InfoTy for usize {
    fn combine(&self, other: &usize) -> usize {
        *self + *other
    }
}

pub struct NulSpace;

impl IntervalSpace for NulSpace {
    type Info = ();

    fn compute_info(&self, _: Interval) -> () { () }
}

impl<I> fmt::Debug for IntervalSet<I>
where I: IntervalSpace,
      I::Info: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

#[derive(Clone, Debug)]
struct IntervalWrap<T: InfoTy> {
    iv: Interval,
    info: T,
}

impl<T: InfoTy> IntervalWrap<T> {
    fn new<S: IntervalSpace<Info=T>>(iv: Interval, space: &S) -> IntervalWrap<T> {
        IntervalWrap {
            iv: iv,
            info: space.compute_info(iv),
        }
    }

    fn shift_right(&mut self, by: usize) {
        self.iv.shift_right(by);
    }

    fn grow<S: IntervalSpace<Info=T>>(&mut self, add: ISpaceAdd, space: &S) {
        assert!(self.iv.beg <= add.index && add.index <= self.iv.end);
        self.iv.end += add.size;
        self.info = self.info.combine(&space.compute_info(add.into()));
    }

    fn split_off<S: IntervalSpace<Info=T>>(&mut self, from: usize, space: &S) -> Interval {
        let ret = self.iv.split_off(from);
        self.info = space.compute_info(self.iv);
        ret
    }
}

impl<I: IntervalSpace> IntervalSet<I> {
    pub fn new(space: I) -> IntervalSet<I> {
        IntervalSet {
            space: space,
            inner: Vec::new(),
        }
    }

    pub fn get_space(&self) -> &I {
        &self.space
    }

    pub fn add<T>(&mut self, iv: T) where T: Into<Interval> {
        let iv = iv.into();
        if iv.beg == iv.end { return; }
        assert!(iv.beg < iv.end);

        match self.inner.binary_search_by(|e| e.iv.cmp(&iv)) {
            Ok(_) => {},
            Err(i) => {
                let mut new_iv = iv;
                let mut ins_idx = i;
                let prev = if i == 0 { None } else { Some(self.inner[i-1].iv) };
                let next = self.inner.get(i).map(|e| e.iv);
                if prev.map_or(true, |prev| prev.end < iv.beg) &&
                   next.map_or(true, |next| iv.end < next.beg) {
                    // do nothing
                } else if prev.map_or(false, |prev| prev.beg < iv.beg && iv.end <= prev.end) ||
                          next.map_or(false, |next| iv.beg == next.beg && iv.end < next.end) {
                    return;
                } else if prev.map_or(false, |prev| iv.beg <= prev.end && prev.end < iv.end) {
                    new_iv = Interval {
                        beg: prev.unwrap().beg,
                        end: self.purge(i-1, iv.end),
                    };
                    ins_idx = i-1;
                } else if prev.map_or(true, |prev| prev.end < iv.beg) &&
                          next.map_or(false, |next| iv.beg <= next.beg && next.beg < iv.end) {
                    new_iv.end = self.purge(i, iv.end);
                } else {
                    unreachable!("Bug! prev: {:?}, next: {:?}, iv: {:?}", prev, next, iv);
                }
                self.inner.insert(ins_idx, IntervalWrap::new(new_iv, &self.space));
            }
        }
    }

    // Removes all intervals starting from index `idx` till (incl.) the interval containing `end`.
    // Panics if `end` lies to the left of the interval at `idx`.
    // The return value is `end` if no intervals contain `end`, otherwise it is `iv.end` where `iv`
    // is the interval that contains `end`.
    fn purge(&mut self, idx: usize, end: usize) -> usize {
        let mut j = self.inner[idx..].binary_search_by(|e| e.iv.cmp(&Interval::from((end, end)))).unwrap_err();
        assert!(j > 0);
        if self.inner.get(idx + j).map_or(false, |e| e.iv.beg == end) {
            j += 1;
        }
        let end = cmp::max(self.inner[idx + j-1].iv.end, end);
        // TODO optimize? compute info only for the "new parts" and combine them with the ones being extended/replaced
        self.inner.drain(idx..idx + j);
        end
    }

    /// Note: info for shifted intervals are not recomputed. If shifts are not hollow, then info
    ///       for the added parts alone are computed and then they are combined with the info of
    ///       those intervals they are combined with.
    pub fn update_space(&mut self, new_space: I, shifts: Vec<ISpaceAdd>, add_to_set: bool) {
        let mut cumulative_shift = 0;
        let mut shift_iter = shifts.into_iter();
        let mut shift_peek = shift_iter.next();
        let mut split_add = Vec::new(); // only when add_to_set is false
        'iv: for iv_wrap in &mut self.inner {
            let shifted_add;
            loop {
                match shift_peek {
                    Some(shift) => {
                        if iv_wrap.iv.beg > shift.index {
                            cumulative_shift += shift.size;
                            shift_peek = shift_iter.next();
                        } else {
                            iv_wrap.shift_right(cumulative_shift);
                            shifted_add = ISpaceAdd {
                                index: shift.index + cumulative_shift,
                                size: shift.size,
                            };
                            break;
                        }
                    }
                    None => {
                        iv_wrap.shift_right(cumulative_shift);
                        continue 'iv;
                    }
                }
            }
            assert!(iv_wrap.iv.beg <= shifted_add.index);
            if add_to_set {
                if shifted_add.index <= iv_wrap.iv.end {
                    iv_wrap.grow(shifted_add, &new_space)
                }
            } else {
                if iv_wrap.iv.beg == shifted_add.index {
                    iv_wrap.shift_right(shifted_add.size);
                } else if shifted_add.index < iv_wrap.iv.end {
                    let mut iv = iv_wrap.split_off(shifted_add.index, &new_space);
                    iv.shift_right(shifted_add.size);
                    split_add.push(iv);
                }
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
    use super::*;

    // TODO find out why uncommenting this will result in test compile error
    //impl<'a, T, U> PartialEq<&'a [(U, T::Info)]> for IntervalSet<T>
    //where T: IntervalSpace,
    //      T::Info: PartialEq,
    //      U: Into<Interval> + Copy
    //{
    //    fn eq(&self, other: &&'a[(U, T::Info)]) -> bool {
    //        self.inner.len() == other.len() && self.inner.iter().zip(*other).all(|(this, other)| this.iv == other.0.into() && this.info == other.1)
    //    }
    //}

    impl<'a, T> PartialEq<&'a [(usize, usize, T::Info)]> for IntervalSet<T>
    where T: IntervalSpace,
          T::Info: PartialEq
    {
        fn eq(&self, other: &&'a[(usize, usize, T::Info)]) -> bool {
            self.inner.len() == other.len() && self.inner.iter().zip(*other).all(|(this, other)| this.iv == Interval::from((other.0, other.1)) && this.info == other.2)
        }
    }

    #[test]
    fn interval_set_add() {
        let mut ivs = IntervalSet::new(NulSpace);
        ivs.extend(vec![(7, 9), (3, 5)]);
        assert_eq!(ivs, &[(3, 5, ()), (7, 9, ())]);
        ivs.add((5, 7));
        assert_eq!(ivs, &[(3, 9, ())]);
        ivs.extend(vec![(7, 8), (11, 13), (15, 16), (18, 19), (10, 17)]);
        assert_eq!(ivs, &[(3, 9, ()), (10, 17, ()), (18, 19, ())]);
    }

    struct TestSpace;

    impl IntervalSpace for TestSpace {
        type Info = usize;

        fn compute_info(&self, iv: Interval) -> usize {
            iv.end - iv.beg
        }
    }

    #[test]
    fn interval_set_space() {
        let mut ivs = IntervalSet::new(TestSpace);
        ivs.extend(vec![(3, 5), (7, 9)]);
        ivs.update_space(TestSpace, vec![ISpaceAdd { index: 7, size: 2 }], false);
        assert_eq!(ivs, &[(3, 5, 2), (9, 11, 2)]);
        ivs.update_space(TestSpace, vec![ISpaceAdd { index: 9, size: 2 }], true);
        assert_eq!(ivs, &[(3, 5, 2), (9, 13, 4)]);
        ivs.update_space(TestSpace, vec![ISpaceAdd { index: 10, size: 2 }], false);
        assert_eq!(ivs, &[(3, 5, 2), (9, 10, 1), (12, 15, 3)]);
    }
}
