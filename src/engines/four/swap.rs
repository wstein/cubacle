//! Wide-turn swaps: finishing in the standard orientation without a rotation.
//!
//! A wide turn of two layers equals the opposite wide turn of the same
//! direction plus a whole-cube rotation: `Rw` is `Lw` followed by `x`. Doing
//! the opposite turn instead leaves the cube in another orientation, and every
//! later move is then named for the face that now sits where the old one was.
//! The solution is the same in every other respect, so the choice for each
//! wide turn is free, and the 24 orientations the cube can be in after a
//! prefix are enough to track (two choices per wide turn collapse into 24
//! states). This finds the orientation sequence that leaves the cube closest
//! to the standard one, so the closing rotation of up to four turns is
//! usually shorter or gone.
//!
//! New code. Only outer turns and wide turns of two layers occur, as in the
//! rest of the engine; the answer still solves the cube exactly.

use super::rotation::Rotations;
use crate::puzzle::{Face, Facelets, Move, MoveSequence, PuzzleKind, Turn};

/// Turns and orientations of the 4x4x4: 36 moves (6 faces, outer or wide,
/// 3 turns) and 24 rotations.
pub(super) struct Swaps {
    moves: Vec<Move>,
    /// `compose[a][b]`: rotation `a` followed by rotation `b`.
    compose: Vec<Vec<usize>>,
    /// `conj[q][m]`: the move that does in orientation `q` what `m` does in
    /// the standard one.
    conj: Vec<Vec<usize>>,
    /// For a wide move: the rotation `r` with `move = opposite wide turn, r`.
    to_opposite: Vec<usize>,
}

fn invert(m: Move) -> Move {
    let turn = match m.turn {
        Turn::Clockwise => Turn::CounterClockwise,
        Turn::Half => Turn::Half,
        Turn::CounterClockwise => Turn::Clockwise,
    };
    Move { turn, ..m }
}

fn opposite(face: Face) -> Face {
    Face::ALL[(face.index() + 3) % 6]
}

fn turn_index(turn: Turn) -> usize {
    match turn {
        Turn::Clockwise => 0,
        Turn::Half => 1,
        Turn::CounterClockwise => 2,
    }
}

fn index(m: &Move) -> usize {
    m.face.index() * 6 + if m.wide { 3 } else { 0 } + turn_index(m.turn)
}

fn sequence(moves: &[Move]) -> MoveSequence {
    MoveSequence::new(moves.to_vec())
}

/// A few fixed scrambles that tell the 36 moves apart.
fn probes() -> Vec<Facelets> {
    [
        "R U F' Rw D2 Lw' B Uw2 F2 L'",
        "Fw R2 U' Bw D L2 Rw' F U2 B'",
        "Uw' B2 Rw F' L Dw2 R U' Fw B2 D",
    ]
    .iter()
    .map(|text| {
        Facelets::solved(PuzzleKind::Cube4)
            .apply(&text.parse().expect("a fixed scramble"))
            .expect("fits a 4x4x4")
    })
    .collect()
}

impl Swaps {
    pub(super) fn new(rotations: &Rotations) -> Swaps {
        let solved = Facelets::solved(PuzzleKind::Cube4);
        let rots: Vec<Vec<Move>> = rotations.all().to_vec();
        let ids: Vec<Facelets> = rots
            .iter()
            .map(|r| solved.apply(&sequence(r)).expect("rotations fit"))
            .collect();
        let id_of = |cube: &Facelets| ids.iter().position(|x| x == cube).expect("a rotation");
        let compose: Vec<Vec<usize>> = rots
            .iter()
            .map(|a| {
                rots.iter()
                    .map(|b| {
                        let both: Vec<Move> = a.iter().chain(b).copied().collect();
                        id_of(&solved.apply(&sequence(&both)).expect("fits"))
                    })
                    .collect()
            })
            .collect();
        let mut moves = Vec::new();
        for face in Face::ALL {
            for wide in [false, true] {
                for turn in [Turn::Clockwise, Turn::Half, Turn::CounterClockwise] {
                    moves.push(if wide {
                        Move::new(face, 2, true, turn)
                    } else {
                        Move::new(face, 1, false, turn)
                    });
                }
            }
        }
        debug_assert!(moves.iter().enumerate().all(|(i, m)| index(m) == i));
        let probes = probes();
        let effects: Vec<Vec<Facelets>> = moves
            .iter()
            .map(|m| {
                probes
                    .iter()
                    .map(|p| p.apply(&sequence(&[*m])).expect("fits"))
                    .collect()
            })
            .collect();
        let conj: Vec<Vec<usize>> = rots
            .iter()
            .map(|q| {
                moves
                    .iter()
                    .map(|m| {
                        let mut around = q.clone();
                        around.push(*m);
                        around.extend(q.iter().rev().map(|&x| invert(x)));
                        let target: Vec<Facelets> = probes
                            .iter()
                            .map(|p| p.apply(&sequence(&around)).expect("fits"))
                            .collect();
                        effects
                            .iter()
                            .position(|e| *e == target)
                            .expect("a rotation maps a move to a move")
                    })
                    .collect()
            })
            .collect();
        let to_opposite: Vec<usize> = moves
            .iter()
            .map(|m| {
                if !m.wide {
                    return usize::MAX;
                }
                let other = Move {
                    face: opposite(m.face),
                    ..*m
                };
                id_of(&solved.apply(&sequence(&[invert(other), *m])).expect("fits"))
            })
            .collect();
        Swaps {
            moves,
            compose,
            conj,
            to_opposite,
        }
    }

    /// `moves` applied to `cube` leave it solved up to a rotation. Returns
    /// the same solution with some wide turns swapped for their opposites,
    /// followed by the shortest closing rotation found; never longer than
    /// `moves` plus the plain closing rotation.
    pub(super) fn finish(
        &self,
        rotations: &Rotations,
        cube: &Facelets,
        moves: &[Move],
    ) -> Option<Vec<Move>> {
        self.finish_with(rotations, cube, moves, Facelets::is_solved)
    }

    /// As [`Swaps::finish`], for any end condition `goal` that a rotation of
    /// the end state can satisfy (the reduction wants the centers home).
    pub(super) fn finish_with(
        &self,
        rotations: &Rotations,
        cube: &Facelets,
        moves: &[Move],
        goal: impl Fn(&Facelets) -> bool,
    ) -> Option<Vec<Move>> {
        let mut states: Vec<Option<Vec<Move>>> = vec![None; 24];
        states[0] = Some(Vec::new());
        for m in moves {
            let mut next: Vec<Option<Vec<Move>>> = vec![None; 24];
            for (q, state) in states.iter().enumerate() {
                let Some(done) = state else { continue };
                let actual = self.moves[self.conj[q][index(m)]];
                if next[q].is_none() {
                    let mut v = done.clone();
                    v.push(actual);
                    next[q] = Some(v);
                }
                if actual.wide {
                    let r = self.to_opposite[index(&actual)];
                    let q2 = self.compose[r][q];
                    if next[q2].is_none() {
                        let mut v = done.clone();
                        v.push(Move {
                            face: opposite(actual.face),
                            ..actual
                        });
                        next[q2] = Some(v);
                    }
                }
            }
            states = next;
        }
        let mut best: Option<Vec<Move>> = None;
        for done in states.into_iter().flatten() {
            let end = cube.apply(&sequence(&done)).ok()?;
            let Some(closing) = rotations.find(&end, &goal) else {
                continue;
            };
            if best
                .as_ref()
                .is_none_or(|b| done.len() + closing.len() < b.len())
            {
                let mut all = done;
                all.extend_from_slice(closing);
                best = Some(all);
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lcg(state: &mut u64, bound: u64) -> u64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*state >> 33) % bound
    }

    /// A prefix that leaves the cube solved up to a rotation: a rotation,
    /// then some turns and their inverse.
    fn prefix(seed: u64, rots: &Rotations) -> Vec<Move> {
        let mut state = seed;
        let mut out = rots.all()[lcg(&mut state, 24) as usize].clone();
        let n = 3 + lcg(&mut state, 8) as usize;
        let turns: Vec<Move> = (0..n)
            .map(|_| {
                let face = Face::ALL[lcg(&mut state, 6) as usize];
                let turn = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise]
                    [lcg(&mut state, 3) as usize];
                if lcg(&mut state, 2) == 0 {
                    Move::new(face, 1, false, turn)
                } else {
                    Move::new(face, 2, true, turn)
                }
            })
            .collect();
        out.extend(turns.iter().copied());
        out.extend(turns.iter().rev().map(|&m| invert(m)));
        out
    }

    #[test]
    fn swaps_solve_and_never_lengthen() {
        let rots = Rotations::new();
        let swaps = Swaps::new(&rots);
        let solved = Facelets::solved(PuzzleKind::Cube4);
        let mut shorter = 0;
        for seed in 1..300 {
            let moves = prefix(seed, &rots);
            let end = solved.apply(&sequence(&moves)).unwrap();
            let plain = rots.find(&end, Facelets::is_solved).unwrap().len();
            let all = swaps.finish(&rots, &solved, &moves).unwrap();
            assert!(solved.apply(&sequence(&all)).unwrap().is_solved(), "{seed}");
            assert!(all.len() <= moves.len() + plain, "{seed}");
            assert!(all
                .iter()
                .all(|m| (m.depth == 1 && !m.wide) || (m.depth == 2 && m.wide)));
            shorter += usize::from(all.len() < moves.len() + plain);
        }
        assert!(shorter > 0, "some prefixes must profit from a swap");
    }
}
