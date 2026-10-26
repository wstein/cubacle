//! The `canonical` field of a state description: the canonical Orbit64 token
//! (one identifier for a cube and its turned, mirrored and recolored images)
//! with the name of the slot convention the token is written in.

use cubacle::canonical::canonical;
use cubacle::puzzle::Facelets;
use serde::Serialize;

/// `{"token": ..., "schema": "orbit64-Nxn-draft@1"}`.
#[derive(Serialize)]
pub(crate) struct Canonical {
    token: String,
    schema: String,
}

impl Canonical {
    /// The canonical token of a state; `None` when the stickers are not a
    /// cube. The search over the 48 symmetries runs on a blocking thread.
    pub(crate) async fn of(facelets: &Facelets) -> Option<Canonical> {
        let facelets = facelets.clone();
        tokio::task::spawn_blocking(move || {
            let size = facelets.kind().size();
            canonical(&facelets).ok().map(|token| Canonical {
                token: token.to_string(),
                schema: format!("orbit64-{size}x{size}-draft@1"),
            })
        })
        .await
        .ok()
        .flatten()
    }
}
