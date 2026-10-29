//! `solve -v`: a hint line when the start or the target is a known pattern.
//! Information only: the search never looks at it, and nothing is downloaded.

use cubacle::notation::Notation;
use cubacle::patterns;
use cubacle::puzzle::Facelets;

/// The lowercase name of the library pattern `state` matches, `None` when it
/// is solved or nothing matches.
fn name(state: &Facelets, library: &crate::patterns_config::Library) -> Option<String> {
    if state.is_solved() {
        return None;
    }
    let report = patterns::detect_with(state, &library.extra, 0);
    Some(report.matches.first()?.name.to_lowercase())
}

/// The pattern name of `start` for `solve --json -v`; `None` when nothing
/// matches or the configuration cannot be read.
pub(crate) fn start_name(config: Option<&std::path::Path>, start: &Facelets) -> Option<String> {
    let config = crate::patterns_config::load(config).ok()?;
    name(start, &config.library())
}

/// The hint line for `state`, or `None` when it is solved or no library
/// pattern matches.
fn line(
    state: &Facelets,
    label: &str,
    library: &crate::patterns_config::Library,
    notation: Notation,
    index: Option<&patterns::RouteIndex>,
) -> Option<String> {
    if state.is_solved() {
        return None;
    }
    let report = patterns::detect_with(state, &library.extra, 0);
    let found = report.matches.first()?;
    let kind = state.kind();
    let generator = index.and_then(|index| index.lookup(state)).map(|hit| {
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

/// Print the hints for the start and the target to stderr. The name needs no
/// lookup; the known generator is added only when the caller has the route
/// index already (`--pattern-aware`), because building it costs about 0.7 s.
pub(crate) fn print(
    config: Option<&std::path::Path>,
    start: &Facelets,
    target: Option<&Facelets>,
    notation: Notation,
    index: Option<&patterns::RouteIndex>,
) {
    let Ok(config) = crate::patterns_config::load(config) else {
        return;
    };
    let library = config.library();
    for (label, state) in [("", Some(start)), (" (target)", target)] {
        if let Some(text) = state.and_then(|s| line(s, label, &library, notation, index)) {
            eprintln!("{text}");
        }
    }
}
