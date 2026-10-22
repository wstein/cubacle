//! The partial solutions the phases pass on, and the bounded list that keeps
//! the best of phase 1.
//!
//! Ported from cs0x7f/TPR-4x4x4-Solver src/FullCube.java (the fields that
//! carry a partial solution, `compareTo`, `ValueComparator`) and the use of
//! `java.util.PriorityQueue` in src/Search.java, MIT, Copyright (c) 2023 Chen
//! Shuang; this file is AGPL-3.0-or-later, see NOTICE.md.
//!
//! The reference keeps the 500 best phase-1 solutions in a binary heap and
//! reads its array in heap order, then sorts it stably by value. Which of
//! several solutions of equal value is kept, and the order they are tried in,
//! follow from the exact sift rules of `PriorityQueue`, so they are written
//! out here: the answers then agree with the reference.

/// The most moves a partial solution holds (phases 1 and 2).
pub(super) const MAX_MOVES: usize = 48;

/// A partial solution: the moves so far (as move numbers of the model cube)
/// and what the next phase needs to know.
#[derive(Clone, Copy)]
pub(super) struct Cand {
    pub moves: [u8; MAX_MOVES],
    pub len: u8,
    /// A lower bound for the length of the solution through this candidate.
    pub value: i32,
    pub length1: u8,
    pub length2: u8,
    /// Whether the two moves that turn the model cube as a whole follow the
    /// first phase (they are not part of the solution).
    pub add1: bool,
    /// The symmetry between the model cube and the real one.
    pub sym: u8,
    /// Phase 3 start: centre state, wing state and its distance bound.
    pub ct3: u32,
    pub edge: i32,
    pub prun3: i32,
}

impl Cand {
    pub(super) fn empty() -> Cand {
        Cand {
            moves: [0; MAX_MOVES],
            len: 0,
            value: 0,
            length1: 0,
            length2: 0,
            add1: false,
            sym: 0,
            ct3: 0,
            edge: 0,
            prun3: 0,
        }
    }

    pub(super) fn moves(&self) -> &[u8] {
        &self.moves[..usize::from(self.len)]
    }

    /// Adds a move; false if there is no room.
    pub(super) fn push(&mut self, m: usize) -> bool {
        if usize::from(self.len) == MAX_MOVES {
            return false;
        }
        self.moves[usize::from(self.len)] = m as u8;
        self.len += 1;
        true
    }
}

/// `java.util.PriorityQueue` with the reference's comparator: the candidate
/// with the greatest value is at the front.
pub(super) struct Heap {
    q: Vec<Cand>,
}

/// The comparator `c2.value - c1.value`.
fn cmp(a: &Cand, b: &Cand) -> i32 {
    b.value - a.value
}

impl Heap {
    pub(super) fn with_capacity(n: usize) -> Heap {
        Heap {
            q: Vec::with_capacity(n),
        }
    }

    pub(super) fn len(&self) -> usize {
        self.q.len()
    }

    pub(super) fn add(&mut self, x: Cand) {
        let mut k = self.q.len();
        self.q.push(x);
        while k > 0 {
            let parent = (k - 1) >> 1;
            let e = self.q[parent];
            if cmp(&x, &e) >= 0 {
                break;
            }
            self.q[k] = e;
            k = parent;
        }
        self.q[k] = x;
    }

    pub(super) fn poll(&mut self) -> Option<Cand> {
        let result = *self.q.first()?;
        let x = self.q.pop().expect("not empty");
        let n = self.q.len();
        if n > 0 {
            let half = n >> 1;
            let mut k = 0;
            while k < half {
                let mut child = 2 * k + 1;
                let mut c = self.q[child];
                let right = child + 1;
                if right < n && cmp(&c, &self.q[right]) > 0 {
                    child = right;
                    c = self.q[child];
                }
                if cmp(&x, &c) <= 0 {
                    break;
                }
                self.q[k] = c;
                k = child;
            }
            self.q[k] = x;
        }
        Some(result)
    }

    /// The array in heap order, sorted stably by value (the reference's
    /// `toArray` and `Arrays.sort`).
    pub(super) fn into_sorted(self) -> Vec<Cand> {
        let mut q = self.q;
        q.sort_by_key(|c| c.value);
        q
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(value: i32, tag: u8) -> Cand {
        let mut c = Cand::empty();
        c.value = value;
        c.sym = tag;
        c
    }

    #[test]
    fn the_greatest_value_comes_out_first() {
        let mut heap = Heap::with_capacity(8);
        for (i, v) in [5, 9, 1, 7, 9, 3].into_iter().enumerate() {
            heap.add(cand(v, i as u8));
        }
        let order: Vec<i32> = std::iter::from_fn(|| heap.poll())
            .map(|c| c.value)
            .collect();
        assert_eq!(order, [9, 9, 7, 5, 3, 1]);
    }

    #[test]
    fn ties_follow_the_sift_rules_of_java() {
        // Added 4a 4b 4c 4d: no sift moves anything (equal values stay put).
        let mut heap = Heap::with_capacity(4);
        for tag in 0..4 {
            heap.add(cand(4, tag));
        }
        let tags: Vec<u8> = heap.q.iter().map(|c| c.sym).collect();
        assert_eq!(tags, [0, 1, 2, 3]);
        // Polling the front moves the last element to the root and sifts it
        // down only past a strictly greater child.
        let front = heap.poll().unwrap();
        assert_eq!(front.sym, 0);
        let tags: Vec<u8> = heap.q.iter().map(|c| c.sym).collect();
        assert_eq!(tags, [3, 1, 2]);
        // Sorting is stable over the heap order.
        let sorted: Vec<u8> = heap.into_sorted().iter().map(|c| c.sym).collect();
        assert_eq!(sorted, [3, 1, 2]);
    }
}
