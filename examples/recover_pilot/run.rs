//! One case through `recover`, as the command runs it: the search and the
//! fresh solve side by side.

use std::thread;
use std::time::{Duration, Instant};

use cubacle::puzzle::{Facelets, MoveSequence, SolveOptions, Solver};
use cubacle::recover::{
    diagnose_staged, Advice, Finding, Limits, MistakeKind, Plan, Scope, Staged,
};

use crate::cases::{Case, Kind};
use crate::data::PilotCube;
use crate::equiv::same_cube;

/// How well the explanation fits what was injected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// All injected steps named, nothing else.
    Full,
    /// Some injected step named.
    Partial,
    /// Other steps named, but they give the very cube the person holds
    /// (commuting moves): as good an explanation as the injected one.
    Equivalent,
    /// Mistakes named, none of them an injected step, and not the same cube.
    Wrong,
    /// Near the path, or nothing found.
    NoExplanation,
    /// The cube is on the path (mistakes cancelled out).
    OnPath,
}

impl Verdict {
    pub const ALL: [Verdict; 6] = [
        Verdict::Full,
        Verdict::Partial,
        Verdict::Equivalent,
        Verdict::Wrong,
        Verdict::NoExplanation,
        Verdict::OnPath,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Verdict::Full => "full",
            Verdict::Partial => "partial",
            Verdict::Equivalent => "equiv",
            Verdict::Wrong => "wrong",
            Verdict::NoExplanation => "none",
            Verdict::OnPath => "on path",
        }
    }
}

pub struct Search {
    pub verdict: Verdict,
    /// For `Full` and `Partial`: the kinds of the named injected steps agree.
    pub kinds_agree: bool,
    pub stage: &'static str,
    pub time: Duration,
    pub timed_out: bool,
    /// The best way back onto the path: moves to do plus printed moves left.
    pub route_total: Option<usize>,
    pub staged: Option<Staged>,
}

pub struct Outcome {
    pub scoped: Search,
    pub plain: Option<Search>,
    pub fresh_len: Option<usize>,
    pub fresh_time: Duration,
    /// Moves the human still has to make with the advice given.
    pub advised: Option<usize>,
    pub advice: &'static str,
}

fn class(kind: &MistakeKind) -> Option<Kind> {
    match kind {
        MistakeKind::Skipped => Some(Kind::Skip),
        MistakeKind::WrongAmount { .. } => Some(Kind::Amount),
        MistakeKind::WrongLayers { .. } => Some(Kind::Layer),
        MistakeKind::Extra { .. } => Some(Kind::Extra),
        _ => None,
    }
}

fn judge(case: &Case, pilot: &PilotCube, now: &Facelets, staged: &Staged) -> (Verdict, bool) {
    match &staged.diagnosis.finding {
        Finding::OnPath { .. } => (Verdict::OnPath, false),
        Finding::Mistakes { mistakes, .. } => {
            let hits: Vec<_> = mistakes
                .iter()
                .filter_map(|m| {
                    case.mistakes
                        .iter()
                        .find(|i| i.step == m.step)
                        .map(|i| (i, m))
                })
                .collect();
            let agree =
                !hits.is_empty() && hits.iter().all(|(i, m)| class(&m.kind) == Some(i.kind));
            let verdict = if hits.is_empty() {
                if same_cube(
                    &pilot.original,
                    &pilot.solution,
                    &staged.diagnosis.finding,
                    now,
                ) {
                    Verdict::Equivalent
                } else {
                    Verdict::Wrong
                }
            } else if hits.len() == mistakes.len() && hits.len() == case.mistakes.len() {
                Verdict::Full
            } else {
                Verdict::Partial
            };
            (verdict, agree)
        }
        _ => (Verdict::NoExplanation, false),
    }
}

fn search(
    case: &Case,
    pilot: &PilotCube,
    now: &Facelets,
    scope: Option<&Scope>,
    limits: &Limits,
) -> Search {
    let started = Instant::now();
    let staged = diagnose_staged(&pilot.original, now, &pilot.solution, limits, scope);
    let time = started.elapsed();
    match staged {
        Ok(staged) => {
            let (verdict, kinds_agree) = judge(case, pilot, now, &staged);
            let n = staged.diagnosis.solution_len;
            Search {
                verdict,
                kinds_agree,
                stage: staged.stage.describe(),
                time,
                timed_out: staged.diagnosis.timed_out,
                route_total: staged.diagnosis.routes.iter().map(|r| r.total(n)).min(),
                staged: Some(staged),
            }
        }
        Err(_) => Search {
            verdict: Verdict::NoExplanation,
            kinds_agree: false,
            stage: "error",
            time,
            timed_out: false,
            route_total: None,
            staged: None,
        },
    }
}

pub fn run(case: &Case, pilot: &PilotCube, solver: &dyn Solver, max_time: Duration) -> Outcome {
    let now = pilot.original.apply(&case.done).expect("apply");
    let limits = Limits {
        resume_depth: 3,
        two_mistakes: true,
        thorough: false,
        max_time: Some(max_time),
        strays: 2,
    };
    let scope = Scope {
        at_move: case.stop,
        sure_until: None,
        phase_starts: pilot.phase_starts.clone(),
    };
    let mut fresh_time = Duration::ZERO;
    let (mut scoped, fresh) = thread::scope(|s| {
        let handle = s.spawn(|| search(case, pilot, &now, Some(&scope), &limits));
        let t = Instant::now();
        let fresh = solver.solve(&now, &SolveOptions::default());
        fresh_time = t.elapsed();
        (handle.join().expect("search"), fresh.solution().cloned())
    });
    let plain = case
        .also_plain
        .then(|| search(case, pilot, &now, None, &limits));
    let fresh_len = fresh.as_ref().map(MoveSequence::len);
    let (advised, advice) = match scoped.staged.take() {
        None => (fresh_len, "fresh"),
        Some(staged) => {
            let plan = Plan::new(staged.diagnosis.clone(), pilot.solution.clone(), fresh);
            let n = pilot.solution.len();
            let out = match &plan.advice {
                Advice::Recover(route) => (Some(route.total(n)), "recover"),
                Advice::Fresh => (fresh_len, "fresh"),
                Advice::Continue => (Some(n - plan_done(&plan)), "continue"),
                Advice::Nothing => (None, "nothing"),
            };
            scoped.staged = Some(staged);
            out
        }
    };
    Outcome {
        scoped,
        plain,
        fresh_len,
        fresh_time,
        advised,
        advice,
    }
}

fn plan_done(plan: &Plan) -> usize {
    match plan.diagnosis.finding {
        Finding::OnPath { done } => done,
        _ => 0,
    }
}
