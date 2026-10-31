//! Origin: Rust re-implementation of `Orbit64.V2.Registry` (flix-orbit64,
//! Apache-2.0, Werner Stein), following `FORMAT-V2.md`.
//!
//! The registry is an ordered list of puzzle types. Each takes the shortest
//! token length whose unallocated values still hold all its states and starts
//! directly after the entries already placed at that length, so appending an
//! entry never changes an earlier entry's length or offset.

use std::sync::OnceLock;

use super::digits;
use crate::notation::orbit64::big::Big;

/// The cube sizes of the initial registry.
const SIZES: std::ops::RangeInclusive<usize> = 2..=15;

/// How much of a sticker is visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Model {
    /// Stickers carry one of six colors; equal centers are not told apart.
    Color,
    /// Every sticker is distinguishable and its rotation is visible.
    Picture,
}

/// Whether the whole-cube orientation is part of the state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pose {
    /// The state is read after turning the cube to the reference pose.
    Canonical,
    /// The whole-cube orientation is stored with the state.
    Framed,
}

/// One puzzle type with its allocated token range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The registry ID.
    pub id: usize,
    /// The cube size.
    pub n: usize,
    pub model: Model,
    pub pose: Pose,
    /// The token length in characters.
    pub length: usize,
    pub(super) offset: Big,
    pub(super) count: Big,
}

impl Entry {
    /// Whether the value is in the range of this entry.
    fn holds(&self, value: &Big) -> bool {
        let mut end = self.offset.clone();
        end.add(&self.count);
        *value >= self.offset && *value < end
    }
}

/// The types of the initial registry, in registry order.
fn initial_types() -> Vec<(usize, Model, Pose)> {
    let mut types = Vec::new();
    for n in SIZES {
        types.push((n, Model::Color, Pose::Canonical));
        types.push((n, Model::Color, Pose::Framed));
        if n != 2 {
            types.push((n, Model::Picture, Pose::Canonical));
            types.push((n, Model::Picture, Pose::Framed));
        }
    }
    types
}

/// Places the types in order with the allocation rule.
fn allocate(types: &[(usize, Model, Pose)]) -> Vec<Entry> {
    let mut used: Vec<Big> = Vec::new();
    let mut entries = Vec::with_capacity(types.len());
    for (id, &(n, model, pose)) in types.iter().enumerate() {
        let count = digits::state_count(n, model, pose);
        // The least length L >= 1 with used(L) + count <= 64^L.
        let mut capacity = Big::one();
        let mut length = 0;
        loop {
            length += 1;
            capacity.mul_add(64, 0);
            if used.len() < length {
                used.push(Big::zero());
            }
            let mut end = used[length - 1].clone();
            end.add(&count);
            if end <= capacity {
                break;
            }
        }
        let offset = used[length - 1].clone();
        used[length - 1].add(&count);
        entries.push(Entry {
            id,
            n,
            model,
            pose,
            length,
            offset,
            count,
        });
    }
    entries
}

/// Every entry of the registry with its length and offset, in ID order.
pub fn entries() -> &'static [Entry] {
    static ENTRIES: OnceLock<Vec<Entry>> = OnceLock::new();
    ENTRIES.get_or_init(|| allocate(&initial_types()))
}

/// The entry with the given ID.
pub fn entry(id: usize) -> Option<&'static Entry> {
    entries().get(id)
}

/// The entry and rank a token of the given length and value addresses.
pub(super) fn lookup(length: usize, value: &Big) -> Option<(&'static Entry, Big)> {
    let found = entries()
        .iter()
        .find(|entry| entry.length == length && entry.holds(value))?;
    let mut rank = value.clone();
    rank.sub(&found.offset);
    Some((found, rank))
}
