//! `solve -v`: a hint line when the start or the target is a known pattern.
//! Information only: the search never looks at it, and nothing is downloaded.

use cubacle::notation::Notation;
use cubacle::patterns;
use cubacle::puzzle::Facelets;

/// The hint line for `state`, or `None` when it is solved or no library
/// pattern matches.
fn line(
    state: &Facelets,
    label: &str,
    library: &crate::patterns_config::Library,
    notation: Notation,
    slices: bool,
) -> Option<String> {
    if state.is_solved() {
        return None;
    }
    let report = patterns::detect_with(state, &library.extra, 0);
    let found = report.matches.first()?;
    let kind = state.kind();
    let index = patterns::RouteIndex::build(kind, library.all(), &|mv| {
        crate::pattern_aware_cmd::in_alphabet(kind, slices, mv)
    });
    let generator = index.lookup(state).map(|hit| {
        format!(
            " (known generator: {}, {} moves)",
            notation.write(kind, &hit.build),
            hit.build.len()
        )
    });
    Some(format!(
        "pattern{label}: {}{}",
        found.name.to_lowercase(),
        generator.unwrap_or_default()
    ))
}

/// Print the hints for the start and the target to stderr.
pub(crate) fn print(
    config: Option<&std::path::Path>,
    start: &Facelets,
    target: Option<&Facelets>,
    notation: Notation,
    slices: bool,
) {
    let Ok(config) = crate::patterns_config::load(config) else {
        return;
    };
    let library = config.library();
    for (label, state) in [("", Some(start)), (" (target)", target)] {
        if let Some(text) = state.and_then(|s| line(s, label, &library, notation, slices)) {
            eprintln!("{text}");
        }
    }
}
