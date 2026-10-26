//! Pictures for the pattern commands: the drawing code of `solve --icons`
//! and `moves --icons` with a label line above every state.

use cubacle::notation::{ColorScheme, Notation};
use cubacle::patterns::Generator;
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind};

use crate::steps::{end_line, PictureArgs, Pictures, SolutionIcons};

/// The picture options, when `--icons` was given.
pub(super) fn options(
    args: &PictureArgs,
    scheme: Option<ColorScheme>,
    notation: Notation,
) -> Option<SolutionIcons> {
    args.icons.then(|| args.options(scheme, notation))
}

/// The pictures of one run. A `Pictures` is made once per cube size, because
/// it asks the terminal when it is made and the icon height depends on the size.
pub(super) struct Gallery<'a> {
    options: &'a SolutionIcons,
    made: Vec<(usize, Pictures<'a>)>,
}

impl<'a> Gallery<'a> {
    pub(super) fn new(options: &'a SolutionIcons) -> Gallery<'a> {
        Gallery {
            options,
            made: Vec::new(),
        }
    }

    fn at(&mut self, size: usize) -> &Pictures<'a> {
        let at = match self.made.iter().position(|(s, _)| *s == size) {
            Some(at) => at,
            None => {
                self.made.push((size, Pictures::new(size, self.options)));
                self.made.len() - 1
            }
        };
        &self.made[at].1
    }

    /// `label`, then the picture of `state`.
    pub(super) fn state(&mut self, label: &str, state: &Facelets) -> String {
        format!(
            "{label}\n{}",
            self.at(state.kind().size()).start_state(state)
        )
    }

    /// The move icons of `moves`, each drawn on the cube the moves before it leave.
    pub(super) fn steps(
        &mut self,
        start: &Facelets,
        moves: &MoveSequence,
    ) -> Result<String, String> {
        self.at(start.kind().size()).steps(start, moves, &[], false)
    }

    /// The move icons of a generator, drawn on the solved cube it starts from.
    pub(super) fn moves(&mut self, generator: &Generator) -> Result<String, String> {
        let kind = PuzzleKind::of_size(generator.size)
            .ok_or_else(|| format!("no {0}x{0}x{0} cube", generator.size))?;
        self.steps(&Facelets::solved(kind), &generator.sequence()?)
    }

    /// The line `End state: TOKEN` and the picture of the cube the moves reach.
    pub(super) fn end(&mut self, state: &Facelets) -> Result<String, String> {
        let mut out = end_line(state)?;
        out.push_str(&self.at(state.kind().size()).start_state(state));
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icons::GraphicsMode;
    use crate::steps::SolutionIcons;
    use crate::views::Views;
    use cubacle::patterns::{Span, Status};

    fn plain() -> SolutionIcons {
        SolutionIcons {
            scheme: None,
            height: Some(96),
            background: None,
            graphics: GraphicsMode::Plain,
            columns: None,
            views: Views::default(),
            numbers: crate::icons::Numbers::default(),
            show_empty: false,
            notation: Notation::default(),
            target: None,
        }
    }

    fn generator(moves: &str) -> Generator {
        Generator {
            size: 3,
            metric: Span::Outer,
            status: Status::BestKnown,
            length: 2,
            number: 1,
            moves: moves.to_owned(),
        }
    }

    #[test]
    fn a_state_has_its_label_above_a_plain_picture() {
        let options = plain();
        let mut gallery = Gallery::new(&options);
        let text = gallery.state("solved", &Facelets::solved(PuzzleKind::Cube3));
        assert!(text.starts_with("solved\n"));
        assert!(!text.contains('\x1b'));
        assert!(text.lines().count() > 2);
    }

    #[test]
    fn a_generator_draws_one_picture_per_move_and_an_end_state() {
        let options = plain();
        let mut gallery = Gallery::new(&options);
        let moves = gallery.moves(&generator("R U")).unwrap();
        assert!(moves.contains("move R1") || moves.contains('R'), "{moves}");
        assert!(!moves.contains('\x1b'));
        let end = gallery.end(&Facelets::solved(PuzzleKind::Cube3)).unwrap();
        assert!(end.starts_with("End state: "));
    }
}
