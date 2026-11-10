// SPDX-License-Identifier: AGPL-3.0-or-later
//! The explicit picture-token drawing path; solver inputs remain color states.

use super::*;
use cubacle::notation::orbit64::v2::{self, Model};

pub(super) fn read(token: &str, kind: PuzzleKind) -> Result<PictureState, String> {
    let (entry, cube) = v2::decode(token).map_err(|e| format!("--picture: {e}"))?;
    if entry.model != Model::Picture {
        return Err("--picture requires an Orbit64 V2 picture token, not a color token".into());
    }
    if entry.n != kind.size() {
        return Err(format!(
            "--picture is a {0}x{0}x{0}; use --size {0}",
            entry.n
        ));
    }
    v2::facelets::to_picture(entry.n, entry.pose, &cube).map_err(|e| format!("--picture: {e}"))
}

pub(super) fn steps(
    args: &MovesArgs,
    kind: PuzzleKind,
    text: &str,
    moves: &MoveSequence,
    mut state: PictureState,
) -> Result<(), String> {
    if args.group == GroupMode::Axis
        || (args.group == GroupMode::Dots && (text.contains('·') || text.contains('(')))
    {
        return Err("--picture step pictures draw one move per icon; use --group none".into());
    }
    let options = args.pictures.options(args.scheme, args.notation);
    let pictures = Pictures::new(kind.size(), &options);
    let mut style = pictures.style().clone();
    if args.grey {
        style.shading = cubacle::render::Shading::GreyMoving;
    }
    let mut out = String::new();
    for (at, action) in actions_of(text, moves).iter().enumerate() {
        let drawn = state.icon(action, &style).map_err(|e| e.to_string())?;
        let drawn = numbered(drawn, at + 1, args.pictures.numbers, style.backdrop);
        out.push_str(&pictures.picture(&[drawn], &style));
        state = state.apply(action).map_err(|e| e.to_string())?;
    }
    let drawn = args.pictures.views.picture_icons(&state, &style);
    out.push_str(&pictures.picture(&drawn, &style));
    std::io::stdout()
        .lock()
        .write_all(out.as_bytes())
        .map_err(|e| format!("cannot write to standard output: {e}"))
}
