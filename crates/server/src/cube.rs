//! The scramble, random and convert endpoints.

use axum::body::Bytes;
use axum::extract::rejection::{BytesRejection, QueryRejection};
use axum::extract::Query;
use axum::http::HeaderMap;
use axum::Json;
use cubacle::generate;
use cubacle::notation::orbit64;
use cubacle::puzzle::Facelets;
use serde::{Deserialize, Serialize};

use crate::canonical::Canonical;
use crate::error::{ApiError, ApiResult};
use crate::input::read_input;
use crate::request::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScrambleParams {
    pub(crate) moves: Option<u16>,
    pub(crate) size: Option<usize>,
    /// `wca` (default) or `sign`.
    pub(crate) notation: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct ScrambleBody {
    pub(crate) scramble: String,
}

pub(crate) async fn scramble(
    params: Result<Query<ScrambleParams>, QueryRejection>,
) -> ApiResult<ScrambleBody> {
    let Query(params) = params.map_err(|e| ApiError::bad_input(e.body_text()))?;
    let moves = params.moves.unwrap_or(25);
    if !(1..=200).contains(&moves) {
        return Err(ApiError::bad_input("moves must be between 1 and 200"));
    }
    let notation = read_notation(params.notation.as_deref())?;
    let kind = engine_kind(params.size.unwrap_or(3))?;
    Ok(Json(ScrambleBody {
        scramble: notation.write(kind, &generate::scramble(kind, usize::from(moves))),
    }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RandomParams {
    pub(crate) size: Option<usize>,
}

#[derive(Serialize)]
pub(crate) struct CubeBody {
    pub(crate) kind: String,
    pub(crate) facelets: String,
    /// The Orbit64 state token; absent when the stickers are not a cube.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) token: Option<String>,
    /// The canonical Orbit64 token and its schema; absent when the stickers
    /// are not a cube.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) canonical: Option<Canonical>,
}

impl CubeBody {
    async fn of(facelets: &Facelets) -> CubeBody {
        CubeBody {
            kind: kind_name(facelets.kind()),
            facelets: facelets.to_string(),
            token: orbit64::encode(facelets).ok(),
            canonical: Canonical::of(facelets).await,
        }
    }
}

pub(crate) async fn random(
    params: Result<Query<RandomParams>, QueryRejection>,
) -> ApiResult<CubeBody> {
    let Query(params) = params.map_err(|e| ApiError::bad_input(e.body_text()))?;
    let kind = engine_kind(params.size.unwrap_or(3))?;
    let facelets = generate::random_state(kind).expect("the 2x2x2 to 7x7x7 have generators");
    Ok(Json(CubeBody::of(&facelets).await))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ConvertRequest {
    pub(crate) cube: Option<String>,
    /// Moves in WCA, SiGN or SSE notation instead of `cube`.
    pub(crate) scramble: Option<String>,
    /// Cube size of a `scramble`, 2 to 7 (default 3).
    pub(crate) size: Option<usize>,
    pub(crate) scheme: Option<String>,
}

pub(crate) async fn convert(
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> ApiResult<CubeBody> {
    let request: ConvertRequest = read_body(&headers, body)?;
    let facelets = read_input(request.cube, request.scramble, request.size, request.scheme).await?;
    Ok(Json(CubeBody::of(&facelets).await))
}
