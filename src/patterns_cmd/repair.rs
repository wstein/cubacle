//! `patterns identify --repair`: the library pattern a state is "pattern plus a
//! few extra moves", found by `cubacle::patterns::repair`.

use std::time::Duration;

use clap::Args;
use cubacle::notation::Notation;
use cubacle::patterns::{self, Pattern};
use cubacle::puzzle::Facelets;

#[derive(Args)]
pub struct RepairArgs {
    /// Also search for the library pattern the state is "pattern plus a few
    /// extra moves" (single layers and wide blocks, either order, any
    /// orientation), up to K blocks (default 6, at most 7); takes seconds to a
    /// minute. Write `--repair=K`. Without it identify runs no search
    #[arg(
        long,
        value_name = "K",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "6",
        value_parser = clap::value_parser!(u8).range(1..=7)
    )]
    pub(super) repair: Option<u8>,
    /// Memory cap of the repair search in megabytes
    #[arg(long, value_name = "MB", default_value_t = patterns::DEFAULT_TABLE_MEMORY_MB)]
    max_memory: usize,
    /// Time limit of the repair search in seconds
    #[arg(long, value_name = "SECONDS", default_value_t = 60)]
    max_time: u64,
    /// How to write the repair moves: wca (Rw, 3Rw) sign (r, 3r) or sse (TR, T3R)
    #[arg(long, default_value = "wca")]
    notation: Notation,
}

/// What a repair search found.
pub(super) struct Found {
    pub(super) fix: Option<patterns::Repair>,
    outcome: patterns::RepairOutcome,
}

/// Runs the search when `--repair` was given.
pub(super) fn search(
    args: &RepairArgs,
    state: &Facelets,
    library: &crate::patterns_config::Library,
) -> Option<Found> {
    let blocks = args.repair?;
    let all: Vec<&Pattern> = library.all().collect();
    let limits = patterns::RepairLimits {
        max_blocks: usize::from(blocks),
        max_memory_mb: args.max_memory,
        max_time: Duration::from_secs(args.max_time),
    };
    let outcome = patterns::repair(state, &all, &limits);
    Some(Found {
        fix: outcome.found.clone(),
        outcome,
    })
}

/// Puts the distance on the `closest:` entry of the repaired pattern.
pub(super) fn mark_closest(found: &Option<Found>, report: &mut patterns::Report) {
    if let Some(Found { fix: Some(fix), .. }) = found {
        for entry in &mut report.closest {
            if entry.id == fix.id || entry.id == fix.catalogue {
                entry.blocks_away = Some(fix.blocks);
            }
        }
    }
}

pub(super) fn print(args: &RepairArgs, verbose: bool, state: &Facelets, found: &Found) {
    let stats = &found.outcome.stats;
    match &found.fix {
        Some(fix) => {
            let moves = args.notation.write(state.kind(), &fix.moves);
            let label = format!("{} ({})", fix.name, fix.catalogue);
            let frame = if fix.frame.is_empty() {
                String::new()
            } else {
                format!(" turned by {}", fix.frame)
            };
            let (first, second) = match fix.order {
                patterns::RepairOrder::After => (format!("{label}{frame}"), moves),
                patterns::RepairOrder::Before => (moves, format!("{label}{frame}")),
            };
            let plural =
                |n: usize, word: &str| format!("{n} {word}{}", if n == 1 { "" } else { "s" });
            println!(
                "repair: {first} then {second} ({}, {})",
                plural(fix.blocks, "block"),
                plural(fix.moves.len(), "turn")
            );
        }
        None if stats.timed_out => println!(
            "repair: none found within {} moves before the time limit ({} of {} candidates tried)",
            stats.max_blocks, stats.tried, stats.candidates
        ),
        None => println!("repair: none within {} moves", stats.max_blocks),
    }
    if verbose {
        println!(
            "repair details: {} candidates, {} too far for {} blocks, {} tried; stored {} blocks deep ({} states, {} MB); {:.1} s",
            stats.candidates,
            stats.pruned,
            stats.max_blocks,
            stats.tried,
            stats.stored_depth,
            stats.table_states,
            stats.table_mb,
            stats.elapsed.as_secs_f64()
        );
    }
}
