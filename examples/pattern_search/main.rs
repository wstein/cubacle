//! Searches the shortest generator of a pattern and prints a line for
//! `src/patterns/generators.txt`.
//!
//! ```text
//! cargo run --release --example pattern_search -- ID SIZE MODE TURNS [MAX_COST] [SECONDS] [MAX_STATES]
//! ```
//!
//! * `ID`: a pattern id from `patterns.txt`.
//! * `MODE`: `outer` (outer blocks from a face: `R`, `Rw`, `3Rw`), `range`
//!   (any block of adjacent layers, `2-3Rw`), or `sym-outer` / `sym-range`
//!   (the same metrics over macro moves that turn mirror-symmetric layer sets,
//!   see `moves.rs`; a much smaller space that reaches the big cubes).
//! * `TURNS`: `half` (half turns only, a much smaller group) or `all`.
//!
//! The search runs from the solved cube forwards and from the target
//! backwards (`search.rs`), so a result over the full move set is a shortest
//! sequence in that set; the symmetric modes are exact only inside their
//! smaller set and are reported as `best-known`. On even cubes the target may
//! be in any orientation. The result is replayed with the library before it
//! is printed. The only input is the structural definition of the pattern.

mod geometry;
mod moves;
mod search;

use std::time::Duration;

use cubacle::notation::Notation;
use cubacle::patterns;
use cubacle::puzzle::{Facelets, LayerMove, MoveSequence, PuzzleKind};
use cubacle::ranges::write_move;

use search::{Limits, Outcome};

fn colours(state: &Facelets) -> Vec<u8> {
    state.stickers().iter().map(|f| *f as u8).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [id, size, mode, turns, rest @ ..] = &args[..] else {
        eprintln!("usage: pattern_search ID SIZE outer|range|sym-outer|sym-range half|all [MAX_COST] [SECONDS] [MAX_STATES]");
        std::process::exit(2);
    };
    let n: usize = size.parse().expect("size");
    let kind = PuzzleKind::of_size(n).expect("cube size");
    let limits = Limits {
        max_cost: rest.first().map_or(12, |s| s.parse().expect("cost")),
        time: Duration::from_secs(rest.get(1).map_or(600, |s| s.parse().expect("seconds"))),
        max_states: rest
            .get(2)
            .map_or(5_000_000, |s| s.parse().expect("states")),
    };
    let half_only = turns == "half";
    let target = colours(
        &patterns::find(id)
            .expect("pattern id")
            .state(kind)
            .expect("defined for this size"),
    );
    let symmetric = mode.starts_with("sym-");
    let range = mode.ends_with("range");
    let set = if symmetric {
        moves::symmetric(n, range, half_only)
    } else {
        moves::plain(n, range, half_only)
    };
    let solved = colours(&Facelets::solved(kind));
    let targets = geometry::seeds(n, &target);
    let outcome = search::search(&set.steps, vec![solved.clone()], targets.clone(), &limits);
    let Outcome::Found(path, cost) = outcome else {
        let Outcome::NotFound(reason) = outcome else {
            unreachable!()
        };
        eprintln!("{id} {n} {mode} {turns}: {reason}");
        return;
    };
    let layers: Vec<LayerMove> = path.iter().flat_map(|&i| set.text[i].clone()).collect();
    assert_eq!(cost, layers.len(), "cost is the move count");
    let text: Vec<String> = layers
        .iter()
        .map(|m| write_move(Notation::Wca, kind, m))
        .collect();
    let plain: Vec<_> = layers.iter().flat_map(LayerMove::to_moves).collect();
    let built = Facelets::solved(kind)
        .apply(&MoveSequence::new(plain))
        .expect("moves fit");
    assert!(
        targets.contains(&colours(&built)),
        "replay does not give the pattern: {}",
        text.join(" ")
    );
    let status = match (symmetric, half_only) {
        (true, _) => "best-known",
        (false, true) => "half-turn-optimal",
        (false, false) => "optimal",
    };
    let metric = if range { "range" } else { "outer" };
    println!(
        "{id} | {n} | {metric} | {status} | {} | {}",
        layers.len(),
        text.join(" ")
    );
}
