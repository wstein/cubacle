//! `patterns cycles`, `identify` and `find`: commands that read a cube state.

use std::time::Duration;

use cubacle::engines::pattern::{find_generator_with, Cap};
use cubacle::notation::{parse_cube, parse_expression};
use cubacle::patterns;
use cubacle::puzzle::Move;

use super::{FindArgs, IdentifyArgs};

/// `patterns cycles`: a move sequence is followed piece by piece; a cube given
/// as text is read from its colours.
pub(super) fn cycles_state(args: &IdentifyArgs, size: usize) -> Result<(), String> {
    let kind = crate::kind_of(size)?;
    let text = match &args.scramble {
        Some(scramble) => scramble.clone(),
        None => args.cube.join(" "),
    };
    let cube = match &args.scramble {
        Some(_) => None,
        None => parse_cube(&text, args.scheme).ok(),
    };
    let (cycles, state_of_cycles) = match cube {
        Some(cube) => {
            crate::state_check::warn(&cube);
            (patterns::cycles_of_state(&cube)?, Some(cube))
        }
        None => match parse_expression(&text, kind) {
            Ok(moves) => (
                patterns::cycles_of_moves(kind, &moves)?,
                cubacle::puzzle::Facelets::solved(kind).apply(&moves).ok(),
            ),
            Err(_) => return crate::parse_text(&text, args.scheme, size).map(|_| ()),
        },
    };
    print!("{cycles}");
    if let Some(state) = state_of_cycles {
        println!("patternness: {} %", patterns::patternness(&state));
    }
    Ok(())
}

/// `patterns identify`: what the structure of a state says about it.
pub(super) fn identify_state(
    args: &IdentifyArgs,
    verbose: bool,
    size: usize,
    library: &crate::patterns_config::Library,
) -> Result<(), String> {
    let state = match &args.scramble {
        Some(scramble) => crate::scrambled(scramble, crate::kind_of(size)?)?,
        None => crate::parse_text(&args.cube.join(" "), args.scheme, size)?.0,
    };
    crate::state_check::warn(&state);
    let report = patterns::detect_with(&state, &library.extra, args.closest);
    if verbose {
        print!("{}", report.detailed());
    } else {
        print!("{report}");
    }
    print_kind(&patterns::measures(&state));
    for pattern in patterns::identify_imported(&library.extra, &state) {
        println!(
            "imported: {} {} (imported from {})",
            pattern.catalogue,
            pattern.name,
            pattern.origin.as_deref().unwrap_or("?")
        );
    }
    Ok(())
}

/// The kind of a state and the measures it was judged by.
fn print_kind(m: &patterns::Measures) {
    let kind = m.kind();
    println!("kind: {kind} ({})", kind.describe());
    println!(
        "measures: fixed {:.2}, defect symmetry {:.2}, moved {} of {} pieces, symmetry {:.2} (order {}), faces {:.2}, simple faces {:.2}, orbits {:.2}",
        m.fixed,
        m.mask_symmetry,
        m.moved,
        m.pieces,
        m.symmetry,
        m.symmetry_order,
        m.faces,
        m.simple_faces,
        m.orbit
    );
}

/// `patterns find`: the moves that build a state from the solved cube.
pub(super) fn find_state(args: &FindArgs, size: usize) -> Result<(), String> {
    let state = match &args.scramble {
        Some(scramble) => crate::scrambled(scramble, crate::kind_of(size)?)?,
        None => crate::parse_text(&args.cube.join(" "), args.scheme, size)?.0,
    };
    crate::state_check::require(&state)?;
    let cap = Cap::megabytes(args.max_memory);
    let time = args.budget_ms.map(Duration::from_millis);
    let alphabet = match &args.moves {
        Some(text) => crate::restricted_cmd::read_set(text, state.kind())?,
        None => None,
    };
    let allow = alphabet.as_ref().map(|a| move |mv: &Move| a.contains(mv));
    let generator = find_generator_with(
        &state,
        cap,
        time,
        allow.as_ref().map(|f| f as &dyn Fn(&Move) -> bool),
    )
    .map_err(|error| error.to_string())?;
    eprintln!("patternness: {} %", patterns::patternness(&state));
    eprintln!("moves {}", generator.moves.len());
    eprintln!(
        "{}",
        crate::metrics_out::line(state.kind(), &generator.moves)
    );
    match (&generator.note, generator.proven) {
        (_, true) => {
            eprintln!("shortest: no sequence of fewer moves exists (searched exhaustively)")
        }
        (Some(note), false) => eprintln!("best known, a shorter one may exist ({note})"),
        (None, false) => eprintln!("best known, a shorter one may exist"),
    }
    println!("{}", args.notation.write(state.kind(), &generator.moves));
    Ok(())
}
