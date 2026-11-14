//! ABI 3: the front-goal solver (`cubacle::engines::three::solve_front_goal`)
//! as a versioned text request and answer.
//!
//! Request (UTF-8, nine fields separated by one TAB, no line break):
//!
//! | # | Field | Meaning |
//! | - | --- | --- |
//! | 1 | `version` | `1`; anything else is refused |
//! | 2 | `facelets` | 54 letters `URFDLB`, the pose in the standard layout, centers home |
//! | 3 | `twists` | six center twists `U,R,F,D,L,B`, each 0 to 3, comma separated |
//! | 4 | `accepted` | nine lists, separated by a space, row-major over the front; each a comma separated list of home facelet numbers 0 to 53 |
//! | 5 | `center_up` | `1` when the center picture must stand upright, else `0` |
//! | 6 | `table_ms` | table build budget in ms, 0 = 3000 |
//! | 7 | `search_ms` | search budget in ms, 0 = 4000 |
//! | 8 | `max_nodes` | search node cap, 0 = 5000000 |
//! | 9 | `max_moves` | move limit, 0 = the engine's own (21) |
//!
//! Answer: the status byte of the result buffer and a text whose first field is
//! the version `1`. Status 0: `1 TAB moves TAB rotation TAB nodes TAB tables_ms`
//! (`moves` in standard notation, `rotation` a word of at most two `x y z` tokens
//! or empty, `tables_ms` the table build time with three decimals). Status 3:
//! `1 TAB nodes=<n>`; status 4: `1 TAB permille=<n>`; statuses 1 and 2: `1 TAB`
//! and a reason. The tables are the nine group tables (`FrontTables::Groups9`).

use std::time::Duration;

use cubacle::engines::three::{
    solve_front_goal_observed, FrontFailure, FrontGoal, FrontLimits, FrontTables,
};
use cubacle::puzzle::{CenterTwists, Facelets, PuzzleKind};

/// Version of the request and the answer text.
pub const FRONT_VERSION: &str = "1";
/// The route is in the text.
pub const FRONT_STATUS_OK: u8 = 0;
/// The request is not accepted; the text says why.
pub const FRONT_STATUS_BAD_INPUT: u8 = 1;
/// No route exists within the move limit, or no state meets the goal.
pub const FRONT_STATUS_NO_ROUTE: u8 = 2;
/// The search time or node budget ended the search; the text holds the nodes.
pub const FRONT_STATUS_SEARCH_BUDGET: u8 = 3;
/// The table build budget ended the build; the text holds its progress.
pub const FRONT_STATUS_TABLE_BUDGET: u8 = 4;

/// Default table build budget in ms.
pub const DEFAULT_TABLE_MS: u64 = 3000;
/// Default search budget in ms.
pub const DEFAULT_SEARCH_MS: u64 = 4000;
/// Default node cap.
pub const DEFAULT_MAX_NODES: u64 = 5_000_000;

fn bad(why: &str) -> (u8, String) {
    (FRONT_STATUS_BAD_INPUT, format!("{FRONT_VERSION}\t{why}"))
}

fn number(field: &str, name: &str) -> Result<u64, String> {
    field.parse().map_err(|_| format!("{name} is not a number"))
}

/// Solves one request; see the module documentation.
#[doc(hidden)]
pub fn front_solve_text(input: &str) -> (u8, String) {
    let f: Vec<&str> = input.split('\t').collect();
    if f[0] != FRONT_VERSION {
        return bad(&format!("unsupported version {:?}, expected 1", f[0]));
    }
    if f.len() != 9 {
        return bad("expected nine tab separated fields");
    }
    let Ok(start) = Facelets::parse(f[1]) else {
        return bad("facelets are not 54 letters of URFDLB");
    };
    let twists: Result<Vec<u8>, _> = f[2].split(',').map(str::parse).collect();
    let Ok(twists) = twists else {
        return bad("twists are not numbers");
    };
    let Ok(twists): Result<[u8; 6], _> = twists.try_into() else {
        return bad("twists are not six numbers");
    };
    let Ok(twists) = CenterTwists::new(PuzzleKind::Cube3, twists) else {
        return bad("twists are not each 0 to 3");
    };
    let groups: Result<Vec<Vec<u8>>, _> = f[3]
        .split(' ')
        .map(|g| g.split(',').map(str::parse).collect())
        .collect();
    let Ok(groups) = groups else {
        return bad("accepted lists are not numbers");
    };
    if groups.len() != 9 {
        return bad("accepted needs nine lists");
    }
    let classes: Vec<&[u8]> = groups.iter().map(Vec::as_slice).collect();
    let center_up = match f[4] {
        "1" => true,
        "0" => false,
        _ => return bad("center_up is 0 or 1"),
    };
    let goal = match FrontGoal::new(&classes, center_up) {
        Ok(goal) => goal,
        Err(error) => return bad(&error.0),
    };
    let mut values = [0u64; 4];
    for (value, (field, name)) in values.iter_mut().zip(f[5..].iter().zip([
        "table_ms",
        "search_ms",
        "max_nodes",
        "max_moves",
    ])) {
        match number(field, name) {
            Ok(n) => *value = n,
            Err(why) => return bad(&why),
        }
    }
    let [table_ms, search_ms, max_nodes, max_moves] = values;
    let mut limits = FrontLimits::default()
        .with_tables(FrontTables::Groups9)
        .with_table_time(Duration::from_millis(if table_ms == 0 {
            DEFAULT_TABLE_MS
        } else {
            table_ms
        }))
        .with_time(Duration::from_millis(if search_ms == 0 {
            DEFAULT_SEARCH_MS
        } else {
            search_ms
        }))
        .with_max_nodes(if max_nodes == 0 {
            DEFAULT_MAX_NODES
        } else {
            max_nodes
        });
    if max_moves > 0 {
        limits = limits.with_max_moves(max_moves.min(255) as u8);
    }
    match solve_front_goal_observed(&start, &twists, &goal, &limits, &mut |_| true) {
        Ok(route) => (
            FRONT_STATUS_OK,
            format!(
                "{FRONT_VERSION}\t{}\t{}\t{}\t{:.3}",
                route.moves,
                route.rotation,
                route.nodes,
                route.tables.as_secs_f64() * 1000.0
            ),
        ),
        Err(FrontFailure::BudgetExhausted { nodes }) => (
            FRONT_STATUS_SEARCH_BUDGET,
            format!("{FRONT_VERSION}\tnodes={nodes}"),
        ),
        Err(FrontFailure::BuildBudgetExhausted { permille }) => (
            FRONT_STATUS_TABLE_BUDGET,
            format!("{FRONT_VERSION}\tpermille={permille}"),
        ),
        Err(FrontFailure::Cancelled { permille }) => (
            FRONT_STATUS_TABLE_BUDGET,
            format!("{FRONT_VERSION}\tpermille={permille}"),
        ),
        Err(FrontFailure::NoRoute(why)) => {
            (FRONT_STATUS_NO_ROUTE, format!("{FRONT_VERSION}\t{why}"))
        }
        Err(FrontFailure::InvalidInput(why)) => bad(&why),
    }
}
