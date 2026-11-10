// SPDX-License-Identifier: AGPL-3.0-or-later
use cubacle::notation::orbit64::v2::{self, Cube, Model, Pose};
use cubacle::render::{PictureState, Style};

fn picture(twist: usize) -> PictureState {
    let mut cube = Cube::solved(3, Model::Picture).unwrap();
    cube.rotations[0] = twist;
    cube.rotations[2] = twist;
    v2::facelets::to_picture(3, Pose::Framed, &cube).unwrap()
}

#[test]
fn fixed_center_twists_are_visible_in_every_view_and_writer() {
    let plain = picture(0);
    let turned = picture(1);
    let style = Style::default();
    for view in ["urf", "folded", "both", "net"] {
        let draw = |p: &PictureState| match view {
            "urf" => vec![p.state_icon(&style)],
            "folded" => vec![p.folded(&style)],
            "both" => p.pair(&style).to_vec(),
            _ => vec![p.net(&style)],
        };
        let a = draw(&plain);
        let b = draw(&turned);
        let svg = |icons: Vec<cubacle::render::Icon>| {
            icons.iter().map(|i| i.to_svg()).collect::<String>()
        };
        let text = |icons: &[cubacle::render::Icon]| {
            icons.iter().map(|i| i.to_text(false)).collect::<String>()
        };
        assert_ne!(text(&a), text(&b), "{view} text loses the twist");
        assert_ne!(svg(a), svg(b), "{view} SVG loses the twist");
    }
}

#[test]
fn rotations_transport_center_tangents_like_the_v2_wire_geometry() {
    let mut cube = Cube::solved(3, Model::Picture).unwrap();
    cube.rotations = vec![1, 2, 3, 0, 1, 1];
    for frame in 0..24 {
        cube.frame = frame;
        let start = v2::facelets::to_picture(3, Pose::Framed, &cube).unwrap();
        for name in ["x", "x'", "x2", "y", "y'", "y2", "z", "z'", "z2"] {
            let actual = start.apply(&name.parse().unwrap()).unwrap();
            let expected = (0..24)
                .find_map(|next| {
                    let mut candidate = cube.clone();
                    candidate.frame = next;
                    let expected = v2::facelets::to_picture(3, Pose::Framed, &candidate).unwrap();
                    (expected.colors() == actual.colors()).then_some(expected)
                })
                .unwrap();
            assert_eq!(actual, expected, "frame {frame}, {name}");
        }
    }
}

#[test]
fn a_layer_turn_rotates_its_center_and_inner_layers_transport_the_others() {
    use cubacle::puzzle::Face;
    let start = picture(0);
    for (name, face, twist) in [("U", Face::U, 1), ("F'", Face::F, 3), ("R2", Face::R, 2)] {
        let after = start.apply(&name.parse().unwrap()).unwrap();
        assert_eq!(after.centers().twist(face), twist);
    }
    let after = start.apply(&"2R".parse().unwrap()).unwrap();
    let inverse = after.apply(&"2R'".parse().unwrap()).unwrap();
    assert_eq!(inverse, start);
    assert_ne!(after.centers(), start.centers());
}

#[test]
fn picture_views_and_rotation_icons_match_svg_and_text_goldens() {
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/picture-render");
    let style = Style::default();
    for (name, twist) in [("upright", 0), ("turned", 1)] {
        let state = picture(twist);
        let [urf, dbl] = state.pair(&style);
        let mut icons = vec![
            ("urf", state.state_icon(&style)),
            ("folded", state.folded(&style)),
            ("net", state.net(&style)),
            ("both-urf", urf),
            ("both-dbl", dbl),
        ];
        for action in ["x", "y", "z"] {
            icons.push((
                action,
                state.icon(&action.parse().unwrap(), &style).unwrap(),
            ));
        }
        for (view, icon) in icons {
            for (extension, actual) in [("svg", icon.to_svg()), ("txt", icon.to_text(false))] {
                let path = directory.join(format!("{name}-{view}.{extension}"));
                let expected = std::fs::read_to_string(&path).unwrap();
                assert_eq!(actual, expected, "{}", path.display());
            }
        }
    }
}

#[test]
fn the_bridge_rejects_invalid_and_unsupported_picture_states() {
    use cubacle::puzzle::{CenterTwists, Facelets, PuzzleKind};
    assert!(PictureState::new(
        Facelets::solved(PuzzleKind::Cube5),
        CenterTwists::untwisted(PuzzleKind::Cube3).unwrap()
    )
    .is_none());
    for n in [2, 4, 6, 9] {
        let cube = Cube::solved(n, Model::Picture).unwrap();
        assert!(matches!(
            v2::facelets::to_picture(n, Pose::Framed, &cube),
            Err(v2::V2Error::Unsupported(_))
        ));
    }
    let mut cube = Cube::solved(3, Model::Picture).unwrap();
    cube.rotations[0] = 1; // Odd total rotation violates the picture-state law.
    assert!(v2::facelets::to_picture(3, Pose::Framed, &cube).is_err());
    for n in [3, 5, 7] {
        let mut cube = Cube::solved(n, Model::Picture).unwrap();
        cube.rotations[0] = 2;
        let state = v2::facelets::to_picture(n, Pose::Framed, &cube).unwrap();
        assert_eq!(
            state.colors(),
            &Facelets::solved(PuzzleKind::of_size(n).unwrap())
        );
        assert_eq!(state.centers().twist(cubacle::puzzle::Face::U), 2);
    }
}

#[test]
fn framed_picture_reference_moves_replay_through_the_renderer_bridge() {
    let mut checked = 0;
    for line in include_str!("fixtures/orbit64_v2_picture_facelets.txt")
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let columns: Vec<_> = line.split('|').collect();
        let entry = v2::entry(columns[0].parse().unwrap()).unwrap();
        if entry.pose != Pose::Framed || ![3, 5, 7].contains(&entry.n) {
            continue;
        }
        let (_, cube) = v2::decode(columns[3]).unwrap();
        let expected = v2::facelets::to_picture(entry.n, entry.pose, &cube).unwrap();
        let mut actual = v2::facelets::to_picture(
            entry.n,
            entry.pose,
            &Cube::solved(entry.n, Model::Picture).unwrap(),
        )
        .unwrap();
        for token in columns[1].split_whitespace() {
            let mut action: cubacle::render::Action = token.parse().unwrap();
            // The reference format names the far outer layer as depth n;
            // the CLI spells that same turn from the opposite face.
            if let cubacle::render::Action::Layer(mv) = &mut action {
                if usize::from(mv.depth) == entry.n {
                    use cubacle::puzzle::{Face, Turn};
                    mv.face = Face::ALL[(mv.face as usize + 3) % 6];
                    mv.depth = 1;
                    mv.turn = match mv.turn {
                        Turn::Clockwise => Turn::CounterClockwise,
                        Turn::CounterClockwise => Turn::Clockwise,
                        Turn::Half => Turn::Half,
                    };
                }
            }
            actual = actual.apply(&action).unwrap();
        }
        assert_eq!(actual, expected, "entry {}, moves {}", entry.id, columns[1]);
        checked += 1;
    }
    assert_eq!(checked, 51);
}
