use crate::puzzle::frame::{Print, Projection, State, Vector};

use super::{canonical, FixedError};

/// Alternative immutable stickers, by physical identity or an opaque printed image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Accept {
    Ids(Vec<u8>),
    Image(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub accept: Accept,
    pub up: Option<Vector>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub p: Vector,
    pub n: Vector,
    pub accept: Vec<Rule>,
}

#[derive(Debug, Clone, Copy, Default)]
struct Geometry {
    p: Vector,
    n: Vector,
    up: Vector,
}

/// A canonical display predicate bound to one immutable reference print.
#[derive(Debug, Clone)]
pub struct FrameGoal {
    print: Print,
    projection: Projection,
    cells: Vec<Cell>,
    masks: Vec<[u32; 8]>,
    dead_rules: Vec<[usize; 2]>,
    canonical: String,
}

fn invalid(message: &str) -> FixedError {
    FixedError::InvalidGoal(message.to_owned())
}

fn axis(vector: Vector) -> bool {
    vector.iter().filter(|v| **v != 0).count() == 1 && vector.iter().all(|v| matches!(v, -1..=1))
}

fn named(print: &Print, accept: &Accept) -> Result<Vec<u8>, FixedError> {
    match accept {
        Accept::Ids(ids) => {
            if ids.is_empty()
                || ids.iter().any(|id| *id >= 24)
                || ids.windows(2).any(|pair| pair[0] == pair[1])
            {
                return Err(invalid("ids must name distinct stickers in 0..23"));
            }
            Ok(ids.clone())
        }
        Accept::Image(image) => {
            if image.chars().count() > 256 {
                return Err(invalid("image exceeds 256 characters"));
            }
            let ids: Vec<_> = print
                .printing()
                .iter()
                .filter(|sticker| sticker.image.as_ref() == Some(image))
                .map(|sticker| sticker.id)
                .collect();
            if ids.is_empty() {
                return Err(invalid("image names no sticker in the print"));
            }
            Ok(ids)
        }
    }
}

impl FrameGoal {
    pub fn new(print: &Print, mut cells: Vec<Cell>) -> Result<FrameGoal, FixedError> {
        if cells.len() > 24 {
            return Err(invalid("a display has at most 24 cells"));
        }
        for cell in &mut cells {
            if !cell.p.iter().all(|v| matches!(v, -1 | 1))
                || !axis(cell.n)
                || cell.p.iter().zip(cell.n).map(|(a, b)| a * b).sum::<i8>() != 1
                || cell.accept.is_empty()
            {
                return Err(invalid(
                    "display cells must be outward facelets with accepted rules",
                ));
            }
            for rule in &mut cell.accept {
                if rule.up.is_some_and(|up| !axis(up)) {
                    return Err(invalid("rule up must be a signed axis"));
                }
                if let Accept::Ids(ids) = &mut rule.accept {
                    ids.sort_unstable();
                }
                named(print, &rule.accept)?;
            }
            cell.accept.sort_by_cached_key(canonical::rule);
            cell.accept.dedup();
        }
        cells.sort_by_key(|cell| (cell.p, cell.n));
        if cells
            .windows(2)
            .any(|pair| (pair[0].p, pair[0].n) == (pair[1].p, pair[1].n))
        {
            return Err(invalid("display cells must be distinct facelets"));
        }
        let mut geometry = [[Geometry::default(); 24]; 24];
        for body in 0..8 {
            for pose in 0..24 {
                let mut partial = [None; 8];
                partial[body] = Some(pose);
                let state = print
                    .complete(partial)
                    .map_err(|e| FixedError::InvalidState(e.to_string()))?;
                for sticker in print
                    .stickers(&state)
                    .map_err(|e| FixedError::InvalidState(e.to_string()))?
                {
                    if sticker.cubie as usize == body {
                        geometry[sticker.id as usize][pose as usize] = Geometry {
                            p: sticker.p,
                            n: sticker.n,
                            up: sticker.up,
                        };
                    }
                }
            }
        }
        let mut bodies = [false; 8];
        let mut masks = vec![[0; 8]; cells.len()];
        let mut dead_rules = Vec::new();
        for (cell_index, cell) in cells.iter().enumerate() {
            for (rule_index, rule) in cell.accept.iter().enumerate() {
                let mut live = false;
                for id in named(print, &rule.accept)? {
                    let body = print.printing()[id as usize].cubie as usize;
                    bodies[body] = true;
                    for (pose, frame) in geometry[id as usize].iter().enumerate() {
                        if frame.p == cell.p
                            && frame.n == cell.n
                            && rule.up.is_none_or(|up| frame.up == up)
                        {
                            masks[cell_index][body] |= 1 << pose;
                            live = true;
                        }
                    }
                }
                if !live {
                    dead_rules.push([cell_index, rule_index]);
                }
            }
        }
        let projection = Projection::new((0..8).filter(|b| bodies[*b]).map(|b| b as u8).collect())
            .map_err(|e| FixedError::InvalidGoal(e.to_string()))?;
        let canonical = canonical::display(&cells);
        Ok(FrameGoal {
            print: print.clone(),
            projection,
            cells,
            masks,
            dead_rules,
            canonical,
        })
    }

    pub fn print(&self) -> &Print {
        &self.print
    }
    pub fn projection(&self) -> &Projection {
        &self.projection
    }
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }
    pub fn dead_rules(&self) -> &[[usize; 2]] {
        &self.dead_rules
    }
    pub fn canonical_display(&self) -> &str {
        &self.canonical
    }

    pub fn matches(&self, state: &State) -> Result<bool, FixedError> {
        self.print
            .state_from_poses(*state.poses())
            .map_err(|e| FixedError::InvalidState(e.to_string()))?;
        Ok(self.matches_poses(state.poses()))
    }

    pub(super) fn matches_poses(&self, poses: &[u8; 8]) -> bool {
        self.masks.iter().all(|mask| {
            self.projection
                .bodies()
                .iter()
                .any(|body| mask[*body as usize] & (1 << poses[*body as usize]) != 0)
        })
    }

    /// Conservative live bytes retained by this goal, including immutable print artwork.
    pub fn memory_bytes(&self) -> usize {
        use std::mem::size_of;
        size_of::<Self>()
            + 8
            + self.cells.capacity() * size_of::<Cell>()
            + self
                .cells
                .iter()
                .map(|cell| {
                    cell.accept.capacity() * size_of::<Rule>()
                        + cell
                            .accept
                            .iter()
                            .map(|rule| match &rule.accept {
                                Accept::Ids(ids) => ids.capacity(),
                                Accept::Image(image) => image.capacity(),
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
            + self.masks.capacity() * size_of::<[u32; 8]>()
            + self.dead_rules.capacity() * size_of::<[usize; 2]>()
            + self.canonical.capacity()
            + self
                .print
                .printing()
                .iter()
                .filter_map(|sticker| sticker.image.as_ref())
                .map(String::capacity)
                .sum::<usize>()
    }
}
