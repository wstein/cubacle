//! Reading the shared parts of a request.

use axum::body::Bytes;
use axum::extract::rejection::BytesRejection;
use axum::http::{header, HeaderMap, StatusCode};
use cubacle::notation::{parse_cube, ColorScheme, Notation};
use cubacle::puzzle::{Facelets, PuzzleKind};

use crate::error::ApiError;

/// The `notation` field of a request: `wca` when absent.
pub(crate) fn read_notation(text: Option<&str>) -> Result<Notation, ApiError> {
    text.map_or(Ok(Notation::Wca), str::parse)
        .map_err(ApiError::bad_input)
}

/// The cube size of a request, if an engine (and so a generator) exists for it.
pub(crate) fn engine_kind(size: usize) -> Result<PuzzleKind, ApiError> {
    match size {
        2..=7 => Ok(PuzzleKind::of_size(size).expect("2 to 7 are sizes")),
        _ => Err(ApiError::bad_input("size must be between 2 and 7")),
    }
}

/// Requires `Content-Type: application/json`. Without this a web page could
/// send a CORS "simple request" (`text/plain`, no preflight) and make a local
/// server burn CPU on its behalf.
pub(crate) fn require_json(headers: &HeaderMap) -> Result<(), ApiError> {
    let essence = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(';').next())
        .map(|v| v.trim().to_ascii_lowercase());
    if essence.as_deref() == Some("application/json") {
        Ok(())
    } else {
        Err(ApiError::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "send the body with Content-Type: application/json",
        ))
    }
}

pub(crate) fn read_body<T: serde::de::DeserializeOwned>(
    headers: &HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Result<T, ApiError> {
    require_json(headers)?;
    let bytes = body.map_err(|e| {
        if e.status() == StatusCode::PAYLOAD_TOO_LARGE {
            ApiError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "payload_too_large",
                "request body is too large",
            )
        } else {
            ApiError::bad_input(e.body_text())
        }
    })?;
    serde_json::from_slice(&bytes).map_err(|e| ApiError::bad_input(format!("bad JSON: {e}")))
}

pub(crate) fn read_cube(cube: &str, scheme: Option<&str>) -> Result<Facelets, ApiError> {
    let scheme = scheme
        .map(str::parse::<ColorScheme>)
        .transpose()
        .map_err(|e| ApiError::bad_input(e.to_string()))?;
    parse_cube(cube, scheme).map_err(|e| ApiError::bad_input(e.to_string()))
}
