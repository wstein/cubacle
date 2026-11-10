// SPDX-License-Identifier: AGPL-3.0-or-later
//! Generates the fixed-center drawing regression fixtures, without a solver.
use cubacle::notation::orbit64::v2::{self, Cube, Model, Pose};
use cubacle::render::{PictureState, Style};
use std::{fs, path::PathBuf};

fn main() {
    let directory = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("usage: picture_render_fixtures OUTPUT_DIRECTORY"),
    );
    fs::create_dir_all(&directory).unwrap();
    let style = Style::default();
    let entry = v2::entries()
        .iter()
        .find(|e| e.n == 3 && e.model == Model::Picture && e.pose == Pose::Framed)
        .unwrap();
    for (name, twist) in [("upright", 0), ("turned", 1)] {
        let mut cube = Cube::solved(3, Model::Picture).unwrap();
        cube.rotations[0] = twist;
        cube.rotations[2] = twist;
        let state: PictureState = v2::facelets::to_picture(3, Pose::Framed, &cube).unwrap();
        println!("{name}: {}", v2::encode(entry.id, &cube).unwrap());
        let [urf, dbl] = state.pair(&style);
        for (view, icon) in [
            ("urf", state.state_icon(&style)),
            ("folded", state.folded(&style)),
            ("net", state.net(&style)),
            ("both-urf", urf),
            ("both-dbl", dbl),
        ] {
            fs::write(directory.join(format!("{name}-{view}.svg")), icon.to_svg()).unwrap();
            fs::write(
                directory.join(format!("{name}-{view}.txt")),
                icon.to_text(false),
            )
            .unwrap();
        }
        for action in ["x", "y", "z"] {
            let icon = state.icon(&action.parse().unwrap(), &style).unwrap();
            fs::write(
                directory.join(format!("{name}-{action}.svg")),
                icon.to_svg(),
            )
            .unwrap();
            fs::write(
                directory.join(format!("{name}-{action}.txt")),
                icon.to_text(false),
            )
            .unwrap();
        }
    }
}
