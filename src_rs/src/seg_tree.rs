pub trait Monoid {
    type Item: Sized + Clone + std::fmt::Debug;
    fn id() -> Self::Item;
    fn op(a: &Self::Item, b: &Self::Item) -> Self::Item;
}

pub enum AddU64 {}

impl Monoid for AddU64 {
    type Item = u64;
    fn id() -> u64 { 0 }
    fn op(a: &u64, b: &u64) -> u64 { a + b }
}

/// 便利な列 `st`
pub enum SegTree<M: Monoid> {
    Leaf {
        val: M::Item,
    },
    Node {
        val: M::Item,
        len: usize,
        left: Box<SegTree<M>>,
        right: Box<SegTree<M>>,
    },
}

impl<M: Monoid> SegTree<M> {
    fn len(&self) -> usize {
        match self {
            Self::Leaf { .. } => 1,
            Self::Node { len, .. } => *len,
        }
    }
    fn val(&self) -> &M::Item {
        match self {
            Self::Leaf { val } => val,
            Self::Node { val, .. } => val,
        }
    }
    /// `st = [M::id(); n]`
    pub fn new(n: usize) -> Self { Self::from(&vec![M::id(); n][..]) }
    pub fn from_slice(slice: &[M::Item]) -> Self {
        if slice.len() == 1 {
            Self::Leaf { val: slice[0].clone() }
        } else {
            let mid = slice.len() / 2;
            let left = Self::from(&slice[.. mid]);
            let right = Self::from(&slice[mid ..]);
            Self::Node {
                len: slice.len(),
                val: M::op(left.val(), right.val()),
                left: Box::new(left),
                right: Box::new(right),
            }
        }
    }
    /// `st[i] = v`
    pub fn set(&mut self, i: usize, v: M::Item) {
        assert!(i < self.len(), "index out: {}/{}", i, self.len());
        match self {
            Self::Leaf { val } => *val = v,
            Self::Node { val, left, right, len, .. } => {
                let mid = *len / 2;
                if i < mid {
                    left.set(i, v);
                } else {
                    right.set(i - mid, v);
                }
                *val = M::op(left.val(), right.val());
            }
        }
    }
    /// `st[i]`
    pub fn get(&self, i: usize) -> &M::Item {
        assert!(i < self.len(), "index out: {}/{}", i, self.len());
        match self {
            Self::Leaf { val } => val,
            Self::Node { left, right, len, .. } => {
                let mid = len / 2;
                if i < mid {
                    left.get(i)
                } else {
                    right.get(i - mid)
                }
            }
        }
    }
    /// `st[range].fold(M::id(), |a, b| M::op(&a, &b))`
    pub fn fold(&self, start: usize, end: usize) -> M::Item {
        assert!(start <= end, "invalid range: {}..{}", start, end);
        assert!(end <= self.len(), "index out: {}/{}", end, self.len());

        let len = end - start;
        if len == 0 {
            return M::id();
        } else if len == self.len() {
            return self.val().clone();
        }

        match self {
            Self::Leaf { .. } => unreachable!(),
            Self::Node { left, right, len, .. } => {
                let mid = len / 2;
                if end <= mid {
                    left.fold(start, end)
                } else if mid <= start {
                    right.fold(start - mid, end - mid)
                } else {
                    M::op(&left.fold(start, mid), &right.fold(0, end - mid))
                }
            }
        }
    }
    /// `pred(st.fold(start..end))` なる最大の `end`
    /// `pred(M::id())` が要請される
    pub fn max_end<P>(&self, start: usize, mut pred: P) -> usize
    where P: FnMut(&M::Item) -> bool {
        assert!(start <= self.len(), "index out: {}/{}", start, self.len());
        let mut acc = M::id();
        self.max_end_inner(start, &mut pred, &mut acc)
    }
    fn max_end_inner<P>(&self, start: usize, pred: &mut P, acc: &mut M::Item) -> usize
    where P: FnMut(&M::Item) -> bool {
        if start == 0 {
            let all_merged = M::op(acc, &self.val());
            if pred(&all_merged) {
                *acc = all_merged;
                return self.len();
            }
        }
        if start == self.len() {
            return self.len();
        }
        match self {
            Self::Leaf { .. } => 0,
            Self::Node { left, right, len, .. } => {
                let mid = len / 2;
                if start < mid {
                    let left_max = left.max_end_inner(start, pred, acc);
                    if left_max < mid {
                        return left_max;
                    }
                }
                mid + right.max_end_inner(start.max(mid) - mid, pred, acc)
            }
        }
    }
    /// `pred(st.fold(start..end))` なる最小の `start`
    /// `pred(M::id())` が要請される
    pub fn min_start<P>(&self, end: usize, mut pred: P) -> usize
    where P: FnMut(&M::Item) -> bool {
        assert!(end <= self.len(), "index out: {}/{}", end, self.len());
        let mut acc = M::id();
        self.min_start_inner(end, &mut pred, &mut acc)
    }
    fn min_start_inner<P>(&self, end: usize, pred: &mut P, acc: &mut M::Item) -> usize
    where P: FnMut(&M::Item) -> bool {
        if end == self.len() {
            let merged = M::op(acc, &self.val());
            if pred(&merged) {
                *acc = merged;
                return 0;
            }
        }
        if end == 0 {
            return 0;
        }
        match self {
            Self::Leaf { .. } => 1,
            Self::Node { left, right, len, .. } => {
                let mid = len / 2;
                if mid <= end {
                    let res_right = right.min_start_inner(end - mid, pred, acc);
                    if res_right > 0 {
                        return mid + res_right;
                    }
                }
                left.min_start_inner(end.min(mid), pred, acc)
            }
        }
    }
}

impl<M: Monoid> From<&[M::Item]> for SegTree<M> {
    fn from(slice: &[M::Item]) -> Self { Self::from_slice(slice) }
}

#[test]
fn test_seg_tree() {
    pub enum M {}
    impl Monoid for M {
        type Item = i32;
        fn id() -> i32 { 0 }
        fn op(a: &i32, b: &i32) -> i32 { a + b }
    }
    let sq = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let st = SegTree::<M>::from(&sq[..]);
    for i in 0 .. sq.len() {
        for j in i .. sq.len() {
            assert_eq!(sq[i .. j].iter().sum::<i32>(), st.fold(i, j))
        }
    }
    for start in 0 ..= sq.len() {
        for max in 0 ..= 55 {
            let mut acc = 0;
            let mut right = start;
            while right < sq.len() && acc + sq[right] <= max {
                acc += sq[right];
                right += 1;
            }
            assert_eq!(st.max_end(start, |&sum| sum <= max), right);
        }
    }
    for end in 0 ..= sq.len() {
        for max in 0 ..= 55 {
            let mut acc = 0;
            let mut left = end;
            while left > 0 && acc + sq[left - 1] <= max {
                left -= 1;
                acc += sq[left];
            }
            assert_eq!(st.min_start(end, |&sum| sum <= max), left, "{} {}", end, max);
        }
    }
}