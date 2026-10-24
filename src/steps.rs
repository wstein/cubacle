//! `solve --icons`: the moves of a solution as icons, in phases and rows.
//!
//! On a Kitty terminal every phase has a heading line, then its icons in rows
//! as wide as the terminal (one image per row, see `rows`), then a framed
//! picture of the cube after the phase. Elsewhere there is one text picture per
//! move, as before.

use std::io::Write;

use cubacle::notation::{orbit64, ColourScheme, Notation};
use cubacle::phases::Phase;
use cubacle::puzzle::{Facelets, Move, MoveSequence, Turn};
use cubacle::render::{icon, Action, Icon, Raster, Style};

use crate::icons::{
    default_height, detect_stdout, show, style, Background, Graphics, GraphicsMode, Numbers,
};
use crate::terminal::{self, Terminal};
use crate::views::{kitty_state, Views};

mod groups;
mod kitty;
pub use groups::Layout;

/// What `solve --icons` was asked for.
pub struct SolutionIcons {
    pub scheme: Option<ColourScheme>,
    /// Icon height in pixels.
    pub height: Option<u32>,
    /// `None`: ask the terminal.
    pub background: Option<Background>,
    /// How the pictures are drawn.
    pub graphics: GraphicsMode,
    /// Icons per row; `None`: as many as fit.
    pub columns: Option<usize>,
    /// Which corners the whole-cube pictures are drawn from.
    pub views: Views,
    /// Whether the icons carry their move number.
    pub numbers: Numbers,
    /// How the move lines and the final solution line write moves (the icon
    /// labels stay as they are).
    pub notation: Notation,
    /// The state the solution leads to when it is not the solved cube.
    pub target: Option<Facelets>,
}

/// The line above the icons of a phase, with the numbers of its moves in the
/// whole solution of `total` moves: `Phase 2: edges (14 moves, moves 9-22 of
/// 27)`, `Phase 3: last layer (1 move, move 27 of 27)`. A phase that is the
/// whole solution says only `Phase 1: solve (3 moves)`; an empty one
/// `Phase 1: solve (nothing to do)`. The numbers are those of the icons and of
/// `recover --at-move`.
pub(crate) fn phase_heading(phase: &Phase, total: usize) -> String {
    let count = phase.moves.len();
    if count == 0 {
        return format!("Phase {}: {} (nothing to do)", phase.number, phase.name);
    }
    let unit = if count == 1 { "move" } else { "moves" };
    let place = if count == total {
        String::new()
    } else if count == 1 {
        format!(", move {} of {total}", phase.moves.start + 1)
    } else {
        format!(
            ", moves {}-{} of {total}",
            phase.moves.start + 1,
            phase.moves.end
        )
    };
    format!(
        "Phase {}: {} ({count} {unit}{place})",
        phase.number, phase.name
    )
}

/// The heading of `found[at]`. With `collapse`, a run of empty phases gets one
/// line, `Phases 1-5: nothing to do`, on its first phase and none on the
/// others (`None`); a lone empty phase keeps its own heading.
pub(crate) fn heading_at(
    found: &[Phase],
    at: usize,
    collapse: bool,
    total: usize,
) -> Option<String> {
    let phase = &found[at];
    if !collapse || !phase.moves.is_empty() {
        return Some(phase_heading(phase, total));
    }
    if at > 0 && found[at - 1].moves.is_empty() {
        return None;
    }
    let run = found[at..]
        .iter()
        .take_while(|p| p.moves.is_empty())
        .count();
    if run == 1 {
        return Some(phase_heading(phase, total));
    }
    Some(format!(
        "Phases {}-{}: nothing to do",
        phase.number,
        found[at + run - 1].number
    ))
}

/// The `--verbose` line of a phase: `phase 2: edges, moves 15-27`, or
/// `phase 2: edges, nothing to do`. It says what `phase_heading` says, so
/// `solve` writes it only where the headings are not shown.
pub(crate) fn phase_line(phase: &Phase) -> String {
    if phase.moves.is_empty() {
        return format!("phase {}: {}, nothing to do", phase.number, phase.name);
    }
    format!(
        "phase {}: {}, moves {}-{}",
        phase.number,
        phase.name,
        phase.moves.start + 1,
        phase.moves.end
    )
}

/// The line that gives the start state as an Orbit64 token.
pub(crate) fn start_line(state: &Facelets) -> Result<String, String> {
    let token = orbit64::encode(state).map_err(|error| error.to_string())?;
    Ok(format!("Start state: {token}\n"))
}

/// The line that names the state a solution leads to, as an Orbit64 token.
pub(crate) fn target_line(state: &Facelets) -> Result<String, String> {
    let token = orbit64::encode(state).map_err(|error| error.to_string())?;
    Ok(format!("Target state: {token}\n"))
}

/// The line that gives the cube after a phase as an Orbit64 token, for
/// `cubacle recover --state <start> --current <token>`.
pub(crate) fn token_line(phase: &Phase, state: &Facelets) -> Result<String, String> {
    let token = orbit64::encode(state).map_err(|error| error.to_string())?;
    Ok(format!(
        "State after phase {} as token: {token}\n",
        phase.number
    ))
}

/// The text marker for a quarter turn that stands between two half turns, the
/// easiest one to do as a half turn.
const QUARTER_NOTE: &str = "   <-- quarter turn, not a half turn";

/// The frame colour of a lone quarter turn in the Kitty rows (amber).
#[cfg(feature = "quarter-frame")]
const QUARTER_FRAME: cubacle::render::Rgb = cubacle::render::Rgb(255, 176, 0);

/// The icon of a move as it goes into a Kitty row: with the `quarter-frame`
/// feature a lone quarter turn gets an amber frame, otherwise the icon as is.
#[cfg_attr(not(feature = "quarter-frame"), allow(unused_variables))]
fn row_icon(raster: Raster, lone: bool) -> Raster {
    #[cfg(feature = "quarter-frame")]
    if lone {
        return raster.framed(QUARTER_FRAME);
    }
    raster
}

/// Whether each move is a quarter turn with a half turn right before and right
/// after it.
pub(crate) fn lone_quarters(moves: &[Move]) -> Vec<bool> {
    let half = |mv: Option<&Move>| mv.is_some_and(|m| m.turn == Turn::Half);
    (0..moves.len())
        .map(|i| {
            moves[i].turn != Turn::Half && i > 0 && half(moves.get(i - 1)) && half(moves.get(i + 1))
        })
        .collect()
}

/// The line above the text picture of a move: `move 3 of 9: F`, with a note
/// when the move is a lone quarter turn.
pub(crate) fn move_line(index: usize, total: usize, mv: &str, lone: bool) -> String {
    let note = if lone { QUARTER_NOTE } else { "" };
    format!("move {} of {total}: {mv}{note}\n", index + 1)
}

/// Whether `solve --icons` writes phase headings: only the Kitty rows have them.
pub(crate) fn shows_phase_headings(mode: GraphicsMode) -> bool {
    detect_stdout(mode) == Graphics::Kitty
}

/// The picture options `solve` and `recover` share.
#[derive(clap::Args)]
pub struct PictureArgs {
    /// Show each move with a picture of the cube (graphics on a capable terminal)
    #[arg(long)]
    pub icons: bool,
    /// Icon height in pixels for --icons (default: 32 per cube layer, at least 96)
    #[arg(long, requires = "icons", value_name = "PIXELS")]
    pub icon_size: Option<u32>,
    /// What the --icons pictures are seen on: sets the notation colour and the
    /// dimming (default: ask the terminal, dark when it does not answer)
    #[arg(long, requires = "icons", value_enum)]
    pub background: Option<Background>,
    /// How the --icons pictures are drawn: auto, kitty, ansi or plain; an
    /// explicit value also applies when output is not a terminal
    #[arg(
        long,
        requires = "icons",
        value_enum,
        env = "CUBACLE_GRAPHICS",
        default_value_t
    )]
    pub graphics: GraphicsMode,
    /// Icons per row for --icons (default: as many as fit the terminal width)
    #[arg(long, requires = "icons", value_name = "N", value_parser = clap::value_parser!(u32).range(1..))]
    pub columns: Option<u32>,
    /// Pictures of a whole cube: folded (default), both corners, urf only, the flat net, or all
    #[arg(long, requires = "icons", value_enum, default_value_t)]
    pub views: Views,
    /// Move numbers in the top-left corner of the --icons pictures: on
    /// (default) or off. The number is the position in the move list, the
    /// N of `recover --at-move N`
    #[arg(long, requires = "icons", value_enum, default_value_t)]
    pub numbers: Numbers,
}

impl PictureArgs {
    pub fn options(&self, scheme: Option<ColourScheme>, notation: Notation) -> SolutionIcons {
        SolutionIcons {
            scheme,
            height: self.icon_size,
            background: self.background,
            graphics: self.graphics,
            columns: self.columns.map(|c| c as usize),
            views: self.views,
            numbers: self.numbers,
            notation,
            target: None,
        }
    }
}

/// The terminal, the style and the options of one run, set up once.
pub struct Pictures<'a> {
    graphics: Graphics,
    terminal: Terminal,
    style: Style,
    options: &'a SolutionIcons,
    collapse_empty: bool,
    /// Hand units to draw the moves in (`moves --icons` with `·`).
    layout: Option<Layout>,
}

impl<'a> Pictures<'a> {
    pub fn new(cube_size: usize, options: &'a SolutionIcons) -> Pictures<'a> {
        let graphics = detect_stdout(options.graphics);
        let terminal = if graphics == Graphics::Kitty {
            terminal::probe(options.background.is_none())
        } else {
            Terminal::default()
        };
        let background = options.background.unwrap_or(match terminal.background {
            Some(colour) if colour.is_light() => Background::Light,
            _ => Background::Dark,
        });
        let height = options.height.unwrap_or_else(|| default_height(cube_size));
        let style = style(options.scheme, height, false, background);
        Pictures {
            graphics,
            terminal,
            style,
            options,
            collapse_empty: false,
            layout: None,
        }
    }

    /// Draws the moves in groups instead of one icon per move.
    pub fn grouped(mut self, layout: Layout) -> Pictures<'a> {
        self.layout = Some(layout);
        self
    }

    /// Runs of phases with nothing to do get one heading line (`recover`).
    pub fn collapsing_empty_phases(mut self) -> Pictures<'a> {
        self.collapse_empty = true;
        self
    }

    /// The move pictures of a solution (no start line, no final moves line); with
    /// `with_start` the picture of the start state comes first.
    pub fn steps(
        &self,
        start: &Facelets,
        solution: &MoveSequence,
        found: &[Phase],
        with_start: bool,
    ) -> Result<String, String> {
        if let Some(layout) = &self.layout {
            return groups::steps(self, start, solution, layout);
        }
        if self.graphics == Graphics::Kitty && solution.is_empty() {
            return Ok(if with_start {
                self.start_state(start)
            } else {
                String::new()
            });
        }
        if self.graphics == Graphics::Kitty {
            kitty::kitty_steps(self, start, solution, found, with_start)
        } else {
            let mut out = if with_start {
                self.start_state(start)
            } else {
                String::new()
            };
            out.push_str(&text_steps(
                start,
                solution,
                found,
                self.graphics,
                &self.style,
                self.options.notation,
            )?);
            Ok(out)
        }
    }

    /// The cube as it should be and as it is, the second marked where it
    /// differs from the first.
    pub fn states_against(
        &self,
        should: (&str, &Facelets),
        now: (&str, &Facelets),
    ) -> Result<String, String> {
        let views = self.options.views;
        self.states_with(&[
            (should.0, views.icons(should.1, None, &self.style), should.1),
            (
                now.0,
                views.icons(now.1, Some(should.1), &self.style),
                now.1,
            ),
        ])
    }

    /// The start state in the current view, directly below its token line
    /// `Start state: TOKEN`, with no heading of its own. On a graphics
    /// terminal it is drawn one and a half times as large as a move icon.
    pub fn start_state(&self, start: &Facelets) -> String {
        let style = if self.graphics == Graphics::Kitty {
            Style {
                size: self.style.size * 3 / 2,
                ..self.style.clone()
            }
        } else {
            self.style.clone()
        };
        let drawn = self.options.views.icons(start, None, &style);
        self.picture(&drawn, &style)
    }

    /// The pictures of one state: one image on a graphics terminal, else text.
    fn picture(&self, drawn: &[Icon], style: &Style) -> String {
        if self.graphics == Graphics::Kitty {
            return kitty_state(drawn, style, &self.terminal, self.options.columns);
        }
        let mut out = String::new();
        for view in drawn {
            out.push_str(&show(view, self.graphics, style.size));
            out.push('\n');
        }
        out
    }

    /// Whole-cube pictures, each under its label and followed by its token.
    fn states_with(&self, states: &[(&str, Vec<Icon>, &Facelets)]) -> Result<String, String> {
        let mut out = String::new();
        for (label, drawn, state) in states {
            out.push_str(label);
            out.push('\n');
            out.push_str(&self.picture(drawn, &self.style));
            let token = orbit64::encode(state).map_err(|error| error.to_string())?;
            out.push_str(&format!("token: {token}\n"));
        }
        Ok(out)
    }
}

/// Each move of the solution with the picture of the cube before it.
pub fn print_solution(
    start: &Facelets,
    solution: &MoveSequence,
    found: &[Phase],
    options: &SolutionIcons,
) -> Result<(), String> {
    let pictures = Pictures::new(start.kind().size(), options);
    let mut out = start_line(start)?;
    if let Some(target) = &options.target {
        out.push_str(&target_line(target)?);
    }
    out.push_str(&pictures.steps(start, solution, found, true)?);
    let mut stdout = std::io::stdout().lock();
    stdout
        .write_all(out.as_bytes())
        .and_then(|()| writeln!(stdout, "{}", options.notation.write(start.kind(), solution)))
        .map_err(|e| format!("cannot write to standard output: {e}"))
}

fn text_steps(
    start: &Facelets,
    solution: &MoveSequence,
    found: &[Phase],
    graphics: Graphics,
    style: &Style,
    notation: Notation,
) -> Result<String, String> {
    let mut state = start.clone();
    let total = solution.len();
    let mut out = String::new();
    let lone = lone_quarters(solution.moves());
    for (index, mv) in solution.moves().iter().enumerate() {
        let action = Action::Layer(*mv);
        let drawn = icon(&state, &action, style).map_err(|error| error.to_string())?;
        out.push_str(&move_line(
            index,
            total,
            &notation.write_move(start.kind(), mv),
            lone[index],
        ));
        out.push_str(&show(&drawn, graphics, style.size));
        out.push('\n');
        state = action.apply(&state).map_err(|error| error.to_string())?;
        for phase in found.iter().filter(|p| p.moves.end == index + 1) {
            if !phase.moves.is_empty() {
                out.push_str(&token_line(phase, &state)?);
                if index + 1 < total {
                    out.push('\n');
                }
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verbose_lines_and_headings_agree() {
        let stages = [
            cubacle::puzzle::SolveStage { name: "a", end: 0 },
            cubacle::puzzle::SolveStage { name: "b", end: 4 },
            cubacle::puzzle::SolveStage { name: "c", end: 9 },
        ];
        let found = cubacle::phases::from_stages(&stages, 9).unwrap();
        let lines: Vec<String> = found.iter().map(phase_line).collect();
        assert_eq!(
            lines,
            [
                "phase 1: a, nothing to do",
                "phase 2: b, moves 1-4",
                "phase 3: c, moves 5-9"
            ]
        );
        let headings: Vec<String> = found.iter().map(|p| phase_heading(p, 9)).collect();
        assert_eq!(
            headings,
            [
                "Phase 1: a (nothing to do)",
                "Phase 2: b (4 moves, moves 1-4 of 9)",
                "Phase 3: c (5 moves, moves 5-9 of 9)"
            ]
        );
    }

    #[test]
    fn empty_phases_collapse_into_one_line() {
        let stages = [
            cubacle::puzzle::SolveStage { name: "a", end: 0 },
            cubacle::puzzle::SolveStage { name: "b", end: 0 },
            cubacle::puzzle::SolveStage { name: "c", end: 0 },
            cubacle::puzzle::SolveStage { name: "d", end: 4 },
            cubacle::puzzle::SolveStage { name: "e", end: 4 },
            cubacle::puzzle::SolveStage { name: "f", end: 9 },
        ];
        let found = cubacle::phases::from_stages(&stages, 9).unwrap();
        let lines: Vec<Option<String>> = (0..found.len())
            .map(|i| heading_at(&found, i, true, 9))
            .collect();
        let want = [
            Some("Phases 1-3: nothing to do"),
            None,
            None,
            Some("Phase 4: d (4 moves, moves 1-4 of 9)"),
            Some("Phase 5: e (nothing to do)"),
            Some("Phase 6: f (5 moves, moves 5-9 of 9)"),
        ];
        assert_eq!(lines, want.map(|l| l.map(str::to_owned)));
        assert_eq!(
            heading_at(&found, 0, false, 9).as_deref(),
            Some("Phase 1: a (nothing to do)")
        );
    }

    #[test]
    fn a_quarter_turn_between_half_turns_is_flagged() {
        let seq: MoveSequence = "3Rw2 3Dw2 F 3Uw2 R U R2 B R2 F".parse().unwrap();
        let lone = lone_quarters(seq.moves());
        let flagged: Vec<usize> = (0..lone.len()).filter(|&i| lone[i]).collect();
        assert_eq!(flagged, [2, 7]);
        let line = move_line(2, 10, &seq.moves()[2].to_string(), true);
        assert_eq!(
            line,
            "move 3 of 10: F   <-- quarter turn, not a half turn\n"
        );
        assert_eq!(
            move_line(0, 10, &seq.moves()[0].to_string(), false),
            "move 1 of 10: 3Rw2\n"
        );
    }

    #[cfg(not(feature = "quarter-frame"))]
    #[test]
    fn kitty_rows_have_no_amber_frame_by_default() {
        let plain = Raster {
            width: 40,
            height: 40,
            data: vec![7; 40 * 40 * 4],
        };
        let shown = row_icon(plain.clone(), true);
        assert_eq!(shown.data, plain.data);
    }

    #[cfg(feature = "quarter-frame")]
    #[test]
    fn kitty_rows_frame_a_lone_quarter_turn_with_the_feature() {
        let plain = Raster {
            width: 40,
            height: 40,
            data: vec![7; 40 * 40 * 4],
        };
        assert_eq!(row_icon(plain.clone(), false).data, plain.data);
        let shown = row_icon(plain, true);
        assert_eq!(&shown.data[..4], [255, 176, 0, 255]);
    }

    #[test]
    fn headings_name_the_phase_and_count_its_moves() {
        let phase = Phase {
            number: 2,
            name: "edges",
            moves: 13..27,
        };
        assert_eq!(
            phase_heading(&phase, 27),
            "Phase 2: edges (14 moves, moves 14-27 of 27)"
        );
        let one = Phase {
            number: 1,
            name: "solve",
            moves: 0..1,
        };
        assert_eq!(phase_heading(&one, 1), "Phase 1: solve (1 move)");
        assert_eq!(
            phase_heading(&one, 5),
            "Phase 1: solve (1 move, move 1 of 5)"
        );
    }
}
