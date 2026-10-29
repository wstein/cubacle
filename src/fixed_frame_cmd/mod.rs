//! Application-layer input shared by the planned fixed-frame command and REST routes.
//!
//! This foundation does not add commands or change any existing solver route.

// Stage 4a: command/server wiring follows the approved reader foundation.
#![cfg_attr(
    not(test),
    expect(dead_code, reason = "fixed-frame command wiring is staged")
)]

mod strict_json;
