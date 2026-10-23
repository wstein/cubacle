use super::*;
use std::collections::HashSet;

#[test]
fn no_duplicate_ids_names_or_aliases() {
    let mut seen = HashSet::new();
    for pattern in all() {
        let keys = std::iter::once(&pattern.id)
            .chain(std::iter::once(&pattern.name))
            .chain(&pattern.aliases);
        let own: HashSet<String> = keys.map(|k| squash(k)).collect();
        for key in own {
            assert!(seen.insert(key.clone()), "{key} is used twice");
        }
    }
}

#[test]
fn every_generator_builds_its_definition() {
    for pattern in all() {
        for generator in pattern.generators() {
            let kind = PuzzleKind::of_size(generator.size).expect("a cube size");
            let target = pattern.state(kind).unwrap_or_else(|| {
                panic!("{} has no definition for {}", pattern.id, generator.size)
            });
            let moves = generator.sequence().expect("parsable moves");
            let built = Facelets::solved(kind).apply(&moves).expect("moves fit");
            assert!(
                same_up_to_rotation(kind, &built, &target),
                "{} {}x{}: {}",
                pattern.id,
                generator.size,
                generator.size,
                generator.moves
            );
            let counted = match generator.metric {
                Metric::Outer => moves.len(),
                Metric::Range => generator.moves.split_whitespace().count(),
            };
            assert_eq!(counted, generator.length, "{} length", pattern.id);
        }
    }
}

#[test]
fn every_definition_has_a_generator() {
    for pattern in all() {
        for size in pattern.sizes() {
            assert!(
                pattern.generator(size, None).is_some(),
                "{} {size}",
                pattern.id
            );
        }
    }
}

#[test]
fn lookup_ignores_case_and_punctuation() {
    assert_eq!(
        find("Cube-in-a-Cube").map(|p| p.id.as_str()),
        Some("cube-in-cube")
    );
    assert!(find("no such pattern").is_none());
}

/// Even cubes have no fixed centres, so a pattern is a pattern in any orientation.
fn same_up_to_rotation(kind: PuzzleKind, built: &Facelets, target: &Facelets) -> bool {
    if built == target {
        return true;
    }
    if kind.size() % 2 == 1 {
        return false;
    }
    let n = kind.size() as u8;
    // A whole-cube rotation is the two halves of one axis turned alike.
    let mut frontier = vec![built.clone()];
    let mut seen = HashSet::from([built.to_string()]);
    while let Some(state) = frontier.pop() {
        for face in [Face::R, Face::U] {
            let half = n / 2;
            let a = crate::puzzle::Move::new(face, half, true, Turn::Clockwise);
            let opposite = Face::ALL[(face.index() + 3) % 6];
            let b = crate::puzzle::Move::new(opposite, half, true, Turn::CounterClockwise);
            let next = state.apply(&MoveSequence::new(vec![a, b])).expect("fits");
            if next == *target {
                return true;
            }
            if seen.insert(next.to_string()) {
                frontier.push(next);
            }
        }
    }
    false
}
