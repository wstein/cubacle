//! Choosing between repairing and solving again, and saying it in words.

use std::fmt;

use crate::puzzle::MoveSequence;

use super::{Diagnosis, Finding, Mistake, MistakeKind, Route};

/// What to do next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Advice {
    /// The cube is on the path: carry on with the printed solution.
    Continue,
    /// Do the route's moves, then carry on with the printed solution.
    Repair(Route),
    /// Solve the cube again from where it is.
    Fresh,
    /// Nothing is known: no way back was found and no fresh solution given.
    Nothing,
}

/// A diagnosis weighed against a fresh solution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub diagnosis: Diagnosis,
    pub solution: MoveSequence,
    /// A solution of the current cube from scratch, if one was computed.
    pub fresh: Option<MoveSequence>,
    pub advice: Advice,
}

impl Plan {
    /// Picks the cheaper of the best route and the fresh solution; on a tie the
    /// route wins, because it keeps the printed solution in use. Never picks a
    /// longer one.
    pub fn new(diagnosis: Diagnosis, solution: MoveSequence, fresh: Option<MoveSequence>) -> Plan {
        let n = diagnosis.solution_len;
        let best = diagnosis
            .routes
            .iter()
            .min_by_key(|r| (r.total(n), r.moves.len()))
            .cloned();
        let advice = match (&diagnosis.finding, best, &fresh) {
            (Finding::OnPath { .. }, _, _) => Advice::Continue,
            (_, Some(route), Some(f)) if route.total(n) <= f.len() => Advice::Repair(route),
            (_, _, Some(_)) => Advice::Fresh,
            (_, Some(route), None) => Advice::Repair(route),
            (_, None, None) => Advice::Nothing,
        };
        Plan {
            diagnosis,
            solution,
            fresh,
            advice,
        }
    }

    /// The moves to do now, in the notation of `solve`.
    pub fn moves(&self) -> Option<&MoveSequence> {
        match &self.advice {
            Advice::Repair(route) => Some(&route.moves),
            Advice::Fresh => self.fresh.as_ref(),
            _ => None,
        }
    }
}

fn describe(m: &Mistake) -> String {
    let step = m.step;
    let printed = m.printed.map(|p| p.to_string()).unwrap_or_default();
    match &m.kind {
        MistakeKind::Skipped => format!("step {step} ({printed}) was skipped"),
        MistakeKind::WrongAmount { done } => {
            format!("step {step} ({printed}) was turned as {done} instead")
        }
        MistakeKind::WrongDirection { done } => {
            format!("step {step} ({printed}) was turned the wrong way ({done})")
        }
        MistakeKind::WrongLayers { done } => {
            format!("step {step} ({printed}) turned other layers ({done})")
        }
        MistakeKind::WrongFace { done } => {
            format!("step {step} ({printed}) was done on the wrong face ({done})")
        }
        MistakeKind::Different { done } => {
            format!("step {step} ({printed}) was replaced by {done}")
        }
        MistakeKind::Swapped { next } => format!(
            "steps {step} and {} ({printed} {next}) were done in the wrong order ({next} {printed})",
            step + 1
        ),
        MistakeKind::Repeated => format!("step {step} ({printed}) was done twice"),
        MistakeKind::Extra { done } if m.printed.is_some() => {
            format!("the extra move {done} was made before step {step} ({printed})")
        }
        MistakeKind::Extra { done } => format!("the extra move {done} was made after the last step"),
    }
}

fn step_of(solution: &MoveSequence, done: usize) -> String {
    match solution.moves().get(done) {
        Some(m) => format!("step {} ({m})", done + 1),
        None => "the end".to_owned(),
    }
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = &self.diagnosis;
        let n = d.solution_len;
        match &d.finding {
            Finding::OnPath { done } if *done == 0 => writeln!(
                f,
                "Your cube is still in its starting state: nothing of the solution is done yet."
            )?,
            Finding::OnPath { done } if *done == n => {
                writeln!(f, "Your cube is at the end of the solution.")?
            }
            Finding::OnPath { done } => writeln!(
                f,
                "Your cube is on the path: you are after step {done} of {n}."
            )?,
            Finding::Mistakes {
                mistakes,
                done,
                alternatives,
            } => {
                writeln!(f, "Your cube is off the path of the solution.")?;
                let label = if mistakes.len() == 1 {
                    "Most likely".to_owned()
                } else {
                    format!("Most likely {} mistakes", mistakes.len())
                };
                for (i, m) in mistakes.iter().enumerate() {
                    if mistakes.len() == 1 {
                        writeln!(f, "{label}: {}.", describe(m))?;
                    } else {
                        if i == 0 {
                            writeln!(f, "{label}:")?;
                        }
                        writeln!(f, "  {}. {}.", i + 1, describe(m))?;
                    }
                }
                writeln!(
                    f,
                    "You had done {} of {n} steps when you stopped.",
                    done
                )?;
                if *alternatives > 0 {
                    writeln!(
                        f,
                        "{} other explanations fit as well (moves that commute cannot be told apart); the earliest is shown.",
                        alternatives
                    )?;
                }
            }
            Finding::Near { done, away } => writeln!(
                f,
                "Your cube is off the path of the solution. It is {} from the cube after step {done}; the cause cannot be named (stray moves or several mistakes).",
                plural(*away, "move")
            )?,
            Finding::Unexplained => writeln!(
                f,
                "Your cube is off the path of the solution, and no simple cause was found (three or more mistakes, or more stray moves than the search covers)."
            )?,
        }
        if d.timed_out {
            writeln!(f, "The search stopped at its time limit.")?;
        }
        if let Some(word) = &d.rotation {
            writeln!(
                f,
                "Your cube is held differently from the original: turn the whole cube ({word}) to match it first. The route below assumes the original orientation."
            )?;
        }
        match &self.advice {
            Advice::Continue => {
                let done = match d.finding {
                    Finding::OnPath { done } => done,
                    _ => 0,
                };
                if done < n {
                    writeln!(f, "Continue with {}.", step_of(&self.solution, done))?;
                }
                return Ok(());
            }
            Advice::Nothing => return writeln!(f, "No way back was found."),
            _ => {}
        }
        let best = d.routes.iter().min_by_key(|r| (r.total(n), r.moves.len()));
        if let Some(route) = best {
            let what = if route.undo {
                format!("undo {} by turning", plural(route.moves.len(), "move"))
            } else {
                format!("do {}", plural(route.moves.len(), "move"))
            };
            writeln!(
                f,
                "Repair: {what} {}, then continue with {}. That is {} moves in all.",
                route.moves,
                step_of(&self.solution, route.continue_after),
                route.total(n)
            )?;
        }
        if let Some(fresh) = &self.fresh {
            writeln!(
                f,
                "Fresh: solve your cube again from where it is, {}.",
                plural(fresh.len(), "move")
            )?;
        }
        match &self.advice {
            Advice::Repair(route) => {
                match &self.fresh {
                    Some(fresh) if fresh.len() > route.total(n) => writeln!(
                        f,
                        "Recommended: repair, {} fewer than solving fresh.",
                        plural(fresh.len() - route.total(n), "move")
                    )?,
                    Some(_) => writeln!(
                        f,
                        "Recommended: repair, as short as solving fresh and it keeps your printed solution."
                    )?,
                    None => writeln!(f, "Recommended: repair.")?,
                }
                writeln!(f, "Moves now: {}", route.moves)?;
                writeln!(
                    f,
                    "Then continue with {} of the printed solution.",
                    step_of(&self.solution, route.continue_after)
                )?;
            }
            Advice::Fresh => {
                if let Some(fresh) = &self.fresh {
                    let why = match best {
                        Some(route) => {
                            format!("shorter than the repair ({} moves)", route.total(n))
                        }
                        None => "no way back onto the path was found".to_owned(),
                    };
                    writeln!(f, "Recommended: solve fresh, {why}.")?;
                    writeln!(f, "Moves now: {fresh}")?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
