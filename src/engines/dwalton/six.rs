//! The 6x6x6 centre stages in a row.
//!
//! New code; the stages are in `six_stage1` and `six_stage2`.

use std::time::Instant;

use super::geometry::{catalogue, MoveDef};
use super::six_stage1::{self, Unpaired};
use super::six_stage2::{self as s2, UdPaired};
use super::six_stage5 as s5;
use super::stage::{select, solve, PatStage};
use crate::puzzle::{Facelets, MoveSequence, PuzzleKind};
use crate::reduction::cache::Cache;

/// Bump when a table changes its layout.
const FORMAT: u32 = 1;

pub(super) struct Six {
    defs: Vec<MoveDef>,
    cache: Cache,
    threads: usize,
}

/// One stage's name and moves.
pub(super) type Done = (&'static str, Vec<String>);

impl Six {
    pub(super) fn new(threads: usize) -> Six {
        Six {
            defs: catalogue(PuzzleKind::Cube6),
            cache: Cache::new(crate::reduction::cache::default_dir(), FORMAT),
            threads,
        }
    }

    fn apply(cube: &Facelets, moves: &[String]) -> Facelets {
        let seq: MoveSequence = moves.join(" ").parse().expect("moves of the catalogue");
        cube.apply(&seq).expect("moves fit the cube")
    }

    /// Stages 1 to 4. Returns the cube after them and the moves of each.
    pub(super) fn centers(
        &self,
        cube: &Facelets,
        max6: i32,
        deadline: Option<Instant>,
    ) -> Option<(Facelets, Vec<Done>, bool)> {
        let mut cube = cube.clone();
        let mut done = Vec::new();
        let t = self.threads;

        let used = select(&self.defs, |_| true);
        let table = six_stage1::table(&used, &self.cache, t);
        let stage = PatStage::new(six_stage1::groups(), &used, vec![&table], Some(&Unpaired));
        let m = solve(
            &stage,
            stage.node_of(|s| cube.stickers()[s] as u8),
            t,
            deadline,
        )?;
        cube = Self::apply(&cube, &m);
        done.push(("LR inner-x staged, obliques paired", m));

        let used = select(&self.defs, s2::five_moves);
        let tables = s2::lr_tables(&used, &self.cache, t);
        let stage = PatStage::new(s2::lr_groups(), &used, tables.iter().collect(), None);
        let m = solve(
            &stage,
            stage.node_of(|s| cube.stickers()[s] as u8),
            t,
            deadline,
        )?;
        cube = Self::apply(&cube, &m);
        done.push(("LR centres staged", m));

        let used = select(&self.defs, s2::ud_pair_moves);
        let table = s2::ud_inner_table(&used, &self.cache, t);
        let stage = PatStage::new(s2::ud_groups(), &used, vec![&table], Some(&UdPaired));
        let m = solve(
            &stage,
            stage.node_of(|s| cube.stickers()[s] as u8),
            t,
            deadline,
        )?;
        cube = Self::apply(&cube, &m);
        done.push(("UD inner-x staged, obliques paired", m));

        let used = select(&self.defs, s2::ud_stage_moves);
        let tables = s2::ud_stage_tables(&used, &self.cache, t);
        let stage = PatStage::new(s2::ud_stage_groups(), &used, tables.iter().collect(), None);
        let m = solve(
            &stage,
            stage.node_of(|s| cube.stickers()[s] as u8),
            t,
            deadline,
        )?;
        cube = Self::apply(&cube, &m);
        done.push(("UD centres staged", m));

        let used = select(&self.defs, s5::step50_moves);
        let table = s5::step50_table(&used, &self.cache, t);
        let stage = PatStage::new(s5::step50_groups(), &used, vec![&table], None);
        let m = solve(
            &stage,
            stage.node_of(|s| cube.stickers()[s] as u8),
            t,
            deadline,
        )?;
        cube = Self::apply(&cube, &m);
        done.push(("LR centres reducible", m));

        let used = select(&self.defs, s5::step60_moves);
        let tables = s5::step60_tables(&used);
        let mut stage = PatStage::new(s5::step60_groups(), &used, tables.iter().collect(), None);
        stage.max_length = max6;
        let m = solve(
            &stage,
            stage.node_of(|s| cube.stickers()[s] as u8),
            t,
            deadline,
        );
        let ok = m.is_some();
        let m = m.unwrap_or_default();
        cube = Self::apply(&cube, &m);
        done.push(("UD FB centres reducible", m));
        Some((cube, done, ok))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::scramble;

    #[test]
    fn step60_solves_its_own_scrambles() {
        let six = Six::new(crate::reduction::par::threads());
        let used = select(&six.defs, s5::step60_moves);
        let tables = s5::step60_tables(&used);
        let mut stage = PatStage::new(s5::step60_groups(), &used, tables.iter().collect(), None);
        stage.max_length = 20;
        let names: Vec<String> = used.iter().map(|d| d.name.clone()).collect();
        let mut x = 12345u64;
        for _ in 0..3 {
            let mut seq = Vec::new();
            for _ in 0..14 {
                x = x
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                seq.push(names[(x >> 33) as usize % names.len()].clone());
            }
            let cube = Six::apply(&Facelets::solved(PuzzleKind::Cube6), &seq);
            let n = stage.node_of(|s| cube.stickers()[s] as u8);
            let m = solve(&stage, n, 12, None).expect("solvable");
            let after = Six::apply(&cube, &m);
            assert!(s5::centres_reduced(&after));
        }
    }

    #[test]
    fn centre_stages_reduce_the_centres() {
        let six = Six::new(crate::reduction::par::threads());
        for _ in 0..3 {
            let cube = Facelets::solved(PuzzleKind::Cube6)
                .apply(&scramble(PuzzleKind::Cube6, 80))
                .unwrap();
            let (after, done, ok) = six.centers(&cube, 20, None).expect("stages");
            assert!(ok);
            assert!(s5::centres_reduced(&after));
            let lens: Vec<usize> = done.iter().map(|d| d.1.len()).collect();
            assert!(lens.iter().sum::<usize>() < 80, "{lens:?}");
        }
    }
}
