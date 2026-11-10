// SPDX-License-Identifier: AGPL-3.0-or-later
use cubacle::notation::orbit64::v2::{self, Cube, Model, Pose};
use std::process::Command;

fn token(twist: usize) -> String {
    let entry = v2::entries()
        .iter()
        .find(|e| e.n == 3 && e.model == Model::Picture && e.pose == Pose::Framed)
        .unwrap();
    let mut cube = Cube::solved(3, Model::Picture).unwrap();
    cube.rotations[0] = twist;
    cube.rotations[2] = twist;
    v2::encode(entry.id, &cube).unwrap()
}

#[test]
fn cli_draws_picture_twists_in_text_and_svg() {
    for format in ["text", "svg"] {
        let run = |twist| {
            Command::new(env!("CARGO_BIN_EXE_cubacle"))
                .args([
                    "moves",
                    "x",
                    "--icons",
                    "--picture",
                    &token(twist),
                    "--format",
                    format,
                ])
                .output()
                .unwrap()
        };
        let a = run(0);
        let b = run(1);
        assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
        assert!(b.status.success(), "{}", String::from_utf8_lossy(&b.stderr));
        assert_ne!(a.stdout, b.stdout, "{format} lost center twist");
    }
}

#[test]
fn cli_whole_cube_views_keep_rotated_center_directions() {
    use cubacle::render::Style;
    let (entry, cube) = v2::decode(&token(1)).unwrap();
    let start = v2::facelets::to_picture(entry.n, entry.pose, &cube).unwrap();
    let state = start.apply(&"x".parse().unwrap()).unwrap();
    for view in ["urf", "folded", "both", "net", "all"] {
        let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
            .args([
                "moves",
                "x",
                "--icons",
                "--picture",
                &token(1),
                "--views",
                view,
                "--graphics",
                "plain",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = String::from_utf8(output.stdout).unwrap();
        let style = Style::default();
        let expected = match view {
            "urf" => vec![state.state_icon(&style)],
            "folded" => vec![state.folded(&style)],
            "both" => state.pair(&style).to_vec(),
            "net" => vec![state.net(&style)],
            _ => {
                let [urf, dbl] = state.pair(&style);
                vec![state.net(&style), urf, dbl]
            }
        };
        for icon in expected {
            assert!(text.contains(&icon.to_text(false)), "{view}: {text}");
        }
    }
}

#[test]
fn cli_rejects_wrong_picture_contracts() {
    let color = v2::entries()
        .iter()
        .find(|e| e.n == 3 && e.model == Model::Color && e.pose == Pose::Framed)
        .unwrap();
    let color_token = v2::encode(color.id, &Cube::solved(3, Model::Color).unwrap()).unwrap();
    for (arguments, reason) in [
        (vec!["--picture", "💫"], "--picture:"),
        (vec!["--picture", " AAAAAAAAAAAAAA"], "--picture:"),
        (vec!["--picture", color_token.as_str()], "not a color token"),
        (
            vec!["--picture", "AAAAAAAAAAAAAA", "--from", "R"],
            "cannot be used with",
        ),
        (
            vec!["--picture", "AAAAAAAAAAAAAA", "--size", "5"],
            "use --size 3",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
            .args(["moves", "x", "--icons"])
            .args(arguments)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(reason),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn cli_raster_and_kitty_use_the_picture_polygons() {
    let directory =
        std::env::temp_dir().join(format!("cubacle-picture-cli-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    for format in ["rgba", "kitty"] {
        let run = |twist| {
            let path = directory.join(format!("{format}-{twist}"));
            let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
                .args([
                    "moves",
                    "x",
                    "--icons",
                    "--picture",
                    &token(twist),
                    "--format",
                    format,
                    "--out",
                ])
                .arg(&path)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            std::fs::read(path).unwrap()
        };
        let a = run(0);
        let b = run(1);
        assert_ne!(a, b, "{format} lost the center direction");
        if format == "rgba" {
            assert_eq!(b.len(), 100 * 96 * 4);
        } else {
            assert!(b.windows(3).any(|w| w == b"\x1b_G"));
        }
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn explicit_picture_steps_honor_cumulative_and_keep_independent_starts() {
    use cubacle::render::{PictureState, Style};
    let (entry, cube) = v2::decode(&token(1)).unwrap();
    let start: PictureState = v2::facelets::to_picture(entry.n, entry.pose, &cube).unwrap();
    for cumulative in [false, true] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cubacle"));
        command.args([
            "moves",
            "x",
            "y",
            "z",
            "--icons",
            "--picture",
            &token(1),
            "--format",
            "text",
            "--graphics",
            "plain",
        ]);
        if cumulative {
            command.arg("--cumulative");
        }
        let output = command.output().unwrap();
        assert!(output.status.success());
        let mut state = start.clone();
        let mut expected = String::new();
        for name in ["x", "y", "z"] {
            let action = name.parse().unwrap();
            expected.push_str(
                &state
                    .icon(&action, &Style::default())
                    .unwrap()
                    .to_text(false),
            );
            expected.push('\n');
            if cumulative {
                state = state.apply(&action).unwrap();
            }
        }
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    }
}
