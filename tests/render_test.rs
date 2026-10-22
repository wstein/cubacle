//! SVG golden fixtures and the public API of `cubacle::render`.
//!
//! Fixtures live in `tests/fixtures/icons/`. After a deliberate change to the
//! drawing, regenerate them with `UPDATE_ICONS=1 cargo test --test render_test`
//! and review the diff.

use cubacle::puzzle::{Facelets, PuzzleKind};
use cubacle::render::{icon, Action, Palette, Shading, Style};
use std::fs;
use std::path::PathBuf;

fn svg(kind: PuzzleKind, text: &str) -> String {
    let action: Action = text.parse().unwrap();
    icon(&Facelets::solved(kind), &action, &Style::default())
        .unwrap()
        .to_svg()
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/icons")
        .join(name)
}

fn check(kind: PuzzleKind, text: &str) {
    let name = format!(
        "{}x{}_{}.svg",
        kind.size(),
        kind.size(),
        text.replace('\'', "p")
    );
    let actual = svg(kind, text);
    if std::env::var_os("UPDATE_ICONS").is_some() {
        fs::create_dir_all(fixture("")).unwrap();
        fs::write(fixture(&name), &actual).unwrap();
    }
    let expected = fs::read_to_string(fixture(&name)).unwrap_or_else(|_| panic!("{name} missing"));
    assert!(
        actual == expected,
        "{name} differs; run with UPDATE_ICONS=1 if the change is intended"
    );
}

#[test]
fn three_by_three_icons_match_their_fixtures() {
    for text in [
        "R", "R'", "U", "U'", "L", "D'", "F", "B'", "Rw", "2R", "R2", "x", "y'",
    ] {
        check(PuzzleKind::Cube3, text);
    }
}

#[test]
fn five_by_five_icons_match_their_fixtures() {
    for text in ["R", "2R", "Rw", "3Rw", "B'", "x"] {
        check(PuzzleKind::Cube5, text);
    }
}

#[test]
fn output_is_deterministic_and_well_formed() {
    for kind in PuzzleKind::ALL {
        let text = if kind.size() > 2 { "Rw'" } else { "R'" };
        let a = svg(kind, text);
        assert_eq!(a, svg(kind, text));
        assert!(a.starts_with("<svg ") && a.ends_with("</svg>\n"));
        assert!(!a.contains("NaN") && !a.contains("inf") && !a.contains("-0.00"));
    }
}

#[test]
fn moves_the_cube_lacks_are_rejected() {
    let action: Action = "3R".parse().unwrap();
    let state = Facelets::solved(PuzzleKind::Cube3);
    assert!(icon(&state, &action, &Style::default()).is_err());
}

#[test]
fn the_icon_uses_the_state_and_the_palette() {
    let state = Facelets::solved(PuzzleKind::Cube3);
    let action: Action = "R".parse().unwrap();
    let boy = Style::default();
    let jp = Style {
        palette: Palette::from_letters(['W', 'R', 'G', 'B', 'O', 'Y']),
        ..Style::default()
    };
    // Down and back are never visible, so the two schemes draw the same icon.
    assert_eq!(
        icon(&state, &action, &boy).unwrap().to_svg(),
        icon(&state, &action, &jp).unwrap().to_svg()
    );
    let turned = state.apply(&"R".parse().unwrap()).unwrap();
    assert_ne!(
        icon(&state, &action, &boy).unwrap().to_svg(),
        icon(&turned, &action, &boy).unwrap().to_svg()
    );
    let grey = Style {
        shading: Shading::GreyMoving,
        ..Style::default()
    };
    assert!(icon(&state, &action, &grey).unwrap().to_svg().contains("#"));
}

#[test]
fn rasters_have_the_requested_height() {
    let state = Facelets::solved(PuzzleKind::Cube4);
    let action: Action = "Rw".parse().unwrap();
    let raster = icon(&state, &action, &Style::default())
        .unwrap()
        .to_rgba(64);
    assert_eq!((raster.width, raster.height), (96, 64));
    assert_eq!(raster.data.len(), 96 * 64 * 4);
    assert!(raster.data.chunks(4).any(|px| px[3] == 255));
    assert!(raster.data.chunks(4).any(|px| px[3] == 0));
}

/// Nothing (stickers, shadow, arrow halo, badge, label) may come within 2 px of
/// the border, whatever the cube size, move or icon size.
#[test]
fn drawings_keep_a_margin_from_the_border() {
    for kind in PuzzleKind::ALL {
        for text in [
            "R", "D'", "L", "U'", "B'", "F", "x", "y'", "z2", "R2", "Rw2", "3Rw'",
        ] {
            let action: Action = text.parse().unwrap();
            if icon(&Facelets::solved(kind), &action, &Style::default()).is_err() {
                continue;
            }
            for size in [64, 96, 128] {
                let drawn = icon(&Facelets::solved(kind), &action, &Style::default()).unwrap();
                let raster = drawn.to_rgba(size);
                let (w, h) = (raster.width as usize, raster.height as usize);
                let (mut left, mut top, mut right, mut bottom) = (w, h, w, h);
                for y in 0..h {
                    for x in 0..w {
                        if raster.data[(y * w + x) * 4 + 3] > 8 {
                            left = left.min(x);
                            top = top.min(y);
                            right = right.min(w - 1 - x);
                            bottom = bottom.min(h - 1 - y);
                        }
                    }
                }
                let margin = left.min(top).min(right).min(bottom);
                assert!(margin >= 2, "{kind:?} {text} at {size}px: margin {margin}");
            }
        }
    }
}
