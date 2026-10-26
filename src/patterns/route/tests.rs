use super::*;
use crate::patterns;

fn index(size: usize) -> RouteIndex {
    RouteIndex::build(
        PuzzleKind::of_size(size).unwrap(),
        patterns::all().iter(),
        &|_| true,
    )
}

#[test]
fn every_image_of_every_builtin_pattern_is_found_and_solved() {
    for size in 3..=5 {
        let kind = PuzzleKind::of_size(size).unwrap();
        let index = index(size);
        assert!(!index.is_empty(), "size {size}");
        let solved = Facelets::solved(kind);
        for pattern in patterns::all() {
            for g in pattern.generators().iter().filter(|g| g.size == size) {
                let sequence = g.sequence().unwrap();
                let state = solved.apply(&sequence).unwrap();
                let colors: Vec<u8> = state.stickers().iter().map(|f| *f as u8).collect();
                for (n, &sym) in symmetries().iter().enumerate() {
                    let key = turned_key(kind, &colors, n, sym);
                    let image = Facelets::from_stickers(
                        key.iter().map(|&c| Face::ALL[c as usize]).collect(),
                    )
                    .unwrap();
                    if image.is_solved() {
                        continue;
                    }
                    let hit = index
                        .lookup(&image)
                        .unwrap_or_else(|| panic!("{} image {n} on {size}", pattern.id));
                    assert!(image.apply(&hit.solve).unwrap().is_solved());
                    assert!(hit.solve.len() <= sequence.len());
                }
            }
        }
    }
}

#[test]
fn a_scramble_and_the_solved_cube_are_not_found() {
    let index = index(3);
    let cube = Facelets::solved(PuzzleKind::Cube3)
        .apply(&"R U F' L2 D B R' U2 F".parse().unwrap())
        .unwrap();
    assert_eq!(index.lookup(&cube), None);
    assert_eq!(index.lookup(&Facelets::solved(PuzzleKind::Cube3)), None);
}

#[test]
fn building_the_builtin_index_is_fast() {
    let started = std::time::Instant::now();
    let index = index(3);
    eprintln!(
        "route index 3x3: {} patterns in {:?}",
        index.len(),
        started.elapsed()
    );
    assert!(started.elapsed().as_secs() < 5);
}
