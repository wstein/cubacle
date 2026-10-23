//! Finding the mistakes that turn the printed solution into the current cube.
//!
//! The person did the printed moves with up to two mistakes (a variant at a
//! position, see [`super::variants`]) and stopped after `q` printed moves. The
//! search works backwards from the current cube: undoing printed moves `q-1`
//! down to `r` gives the cube a person who made a mistake before `r` must have
//! had, for every `(r, q)`. Those few states are looked up against the states
//! the mistakes produce from the printed trajectory.

use std::time::Instant;

use super::state::{hash, MoveId, Perms, State};
use super::variants::Variant;
use super::Mistake;

/// What a frame (the current cube, maybe rotated) is searched against.
pub(super) struct Frame<'a> {
    pub(super) perms: &'a Perms,
    pub(super) printed: &'a [MoveId],
    /// The cube after 0..=n printed moves.
    pub(super) trajectory: &'a [State],
    pub(super) current: State,
    pub(super) deadline: Option<Instant>,
}

impl Frame<'_> {
    pub(super) fn expired(&self) -> bool {
        self.deadline.is_some_and(|d| Instant::now() >= d)
    }

    /// The cube after the printed moves `from..to`, starting from `state`.
    fn walk(&self, state: &[u8], from: usize, to: usize) -> State {
        self.perms.apply_all(state, &self.printed[from..to])
    }
}

/// The mistakes found, with the moves actually done.
pub(super) struct Explained {
    pub(super) mistakes: Vec<Mistake>,
    /// The printed moves done correctly before the first mistake.
    pub(super) branch: usize,
    /// The printed moves used up when the cube was entered.
    pub(super) done: usize,
    /// Everything done after `branch`, in order.
    pub(super) performed: Vec<MoveId>,
    /// Stray moves made after the last printed move used (part of `performed`).
    pub(super) strays: Vec<MoveId>,
    /// Other explanations of equal plausibility.
    pub(super) alternatives: usize,
}

/// The cube whose position in the printed solution is `r`, for every `r <= q`.
struct Backward {
    /// `(key, r, q)` sorted by key.
    entries: Vec<(u64, u16, u16)>,
}

impl Backward {
    fn build(frame: &Frame) -> Backward {
        let n = frame.printed.len();
        let mut entries = Vec::with_capacity((n + 1) * (n + 2) / 2);
        for q in 0..=n {
            let mut state = frame.current.clone();
            for r in (0..=q).rev() {
                entries.push((hash(&state, r), r as u16, q as u16));
                if r > 0 {
                    state = frame
                        .perms
                        .apply(&state, frame.perms.inv(frame.printed[r - 1]));
                }
            }
        }
        entries.sort_unstable();
        Backward { entries }
    }

    fn find(&self, key: u64) -> impl Iterator<Item = (usize, usize)> + '_ {
        let lo = self.entries.partition_point(|e| e.0 < key);
        self.entries[lo..]
            .iter()
            .take_while(move |e| e.0 == key)
            .map(|e| (usize::from(e.1), usize::from(e.2)))
    }
}

/// The printed position the cube is exactly at, if any.
pub(super) fn on_path(frame: &Frame) -> Option<usize> {
    frame.trajectory.iter().position(|t| *t == frame.current)
}

/// One mistake and then the printed moves.
pub(super) fn single(frame: &Frame, variants: &[Vec<Variant>]) -> Option<Explained> {
    let n = frame.printed.len();
    let backward = Backward::build(frame);
    let mut hits: Vec<(u8, usize, usize, usize)> = Vec::new();
    for (i, list) in variants.iter().enumerate() {
        if frame.expired() {
            return None;
        }
        for (v, variant) in list.iter().enumerate() {
            let state = frame.perms.apply_all(&frame.trajectory[i], &variant.moves);
            let r = i + variant.span;
            if r > n {
                continue;
            }
            for (_, q) in backward.find(hash(&state, r)).filter(|e| e.0 == r) {
                let after = frame.walk(&state, r, q);
                if after == frame.current {
                    hits.push((variant.score, i, q, v));
                }
            }
        }
    }
    hits.sort_unstable();
    let &(score, i, q, v) = hits.first()?;
    let variant = &variants[i][v];
    let mut performed = variant.moves.clone();
    performed.extend_from_slice(&frame.printed[i + variant.span..q]);
    let alternatives = hits.iter().filter(|h| h.0 == score).count() - 1;
    Some(Explained {
        mistakes: vec![variant.mistake.clone()],
        branch: i,
        done: q,
        performed,
        strays: Vec::new(),
        alternatives,
    })
}

/// One mistake, then the printed moves, then up to `max` stray moves.
pub(super) fn with_strays(
    frame: &Frame,
    variants: &[Vec<Variant>],
    max: usize,
) -> Option<Explained> {
    let n = frame.printed.len();
    let near = super::resume::near_table(frame, &frame.current, max);
    // (strays, score, position, variant, printed moves used, fingerprint)
    let mut hits: Vec<(usize, u8, usize, usize, usize, u64)> = Vec::new();
    for (i, list) in variants.iter().enumerate() {
        if frame.expired() {
            break;
        }
        for (v, variant) in list.iter().enumerate() {
            let mut state = frame.perms.apply_all(&frame.trajectory[i], &variant.moves);
            for b in i + variant.span..=n {
                let wanted = super::resume::key(&state);
                for strays in super::resume::lookup(&near, wanted).filter(|&d| d > 0) {
                    hits.push((strays, variant.score, i, v, b, wanted));
                }
                if b < n {
                    state = frame.perms.apply(&state, frame.printed[b]);
                }
            }
        }
    }
    hits.sort_unstable();
    let &(strays, score, i, v, b, wanted) = hits.first()?;
    let variant = &variants[i][v];
    let walked = super::resume::find_path(frame, &frame.current, strays, wanted);
    let stray_moves = frame.perms.undo_of(&walked);
    let mut performed = variant.moves.clone();
    performed.extend_from_slice(&frame.printed[i + variant.span..b]);
    performed.extend_from_slice(&stray_moves);
    if walked.len() != strays
        || frame.perms.apply_all(&frame.trajectory[i], &performed) != frame.current
    {
        return None;
    }
    let alternatives = hits
        .iter()
        .filter(|h| h.0 == strays && h.1 == score)
        .count()
        - 1;
    Some(Explained {
        mistakes: vec![variant.mistake.clone()],
        branch: i,
        done: b,
        performed,
        strays: stray_moves,
        alternatives,
    })
}

/// Two mistakes with printed moves between and after them.
pub(super) fn double(frame: &Frame, variants: &[Vec<Variant>]) -> Option<Explained> {
    let n = frame.printed.len();
    let spans = super::variants::by_span(variants);
    // Forward: after the first mistake at `a` and the printed moves up to `b`.
    let mut forward: Vec<(u64, u16, u32, u16)> = Vec::new();
    for (a, list) in variants.iter().enumerate() {
        if frame.expired() {
            return None;
        }
        for (v, variant) in list.iter().enumerate() {
            let mut state = frame.perms.apply_all(&frame.trajectory[a], &variant.moves);
            for b in a + variant.span..=n {
                forward.push((hash(&state, b), a as u16, v as u32, b as u16));
                if b < n {
                    state = frame.perms.apply(&state, frame.printed[b]);
                }
            }
        }
    }
    forward.sort_unstable();
    // Backward: undo the second mistake at `b` from the cube at `r`.
    let mut hits: Vec<(u8, usize, usize, usize, usize, usize)> = Vec::new();
    for q in 0..=n {
        if frame.expired() {
            return None;
        }
        let mut state = frame.current.clone();
        for r in (0..=q).rev() {
            for b in r.saturating_sub(2)..=r {
                for &w in &spans[b][r - b] {
                    let second = &variants[b][w];
                    let mut before = state.clone();
                    for &m in second.moves.iter().rev() {
                        before = frame.perms.apply(&before, frame.perms.inv(m));
                    }
                    let key = hash(&before, b);
                    let lo = forward.partition_point(|e| e.0 < key);
                    for e in forward[lo..].iter().take_while(|e| e.0 == key) {
                        let (a, v) = (usize::from(e.1), e.2 as usize);
                        let first = &variants[a][v];
                        hits.push((first.score + second.score, a, b, v, w, q));
                    }
                }
            }
            if r > 0 {
                state = frame
                    .perms
                    .apply(&state, frame.perms.inv(frame.printed[r - 1]));
            }
        }
        if hits.len() > 20_000 {
            break;
        }
    }
    hits.sort_unstable();
    let mut verified = Vec::new();
    for &(score, a, b, v, w, q) in &hits {
        let (first, second) = (&variants[a][v], &variants[b][w]);
        let mut performed = first.moves.clone();
        performed.extend_from_slice(&frame.printed[a + first.span..b]);
        performed.extend_from_slice(&second.moves);
        performed.extend_from_slice(&frame.printed[b + second.span..q]);
        if frame.perms.apply_all(&frame.trajectory[a], &performed) == frame.current {
            verified.push((score, a, performed, q, [first, second]));
        }
    }
    let (score, a, performed, q, [first, second]) = {
        let best = verified.iter().min_by_key(|v| (v.0, v.1))?;
        (best.0, best.1, best.2.clone(), best.3, best.4)
    };
    let alternatives = verified.iter().filter(|v| v.0 == score).count() - 1;
    Some(Explained {
        mistakes: vec![first.mistake.clone(), second.mistake.clone()],
        branch: a,
        done: q,
        performed,
        strays: Vec::new(),
        alternatives,
    })
}
