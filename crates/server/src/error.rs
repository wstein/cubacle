//! The JSON error of every endpoint and its HTTP mapping.

use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// A JSON error with its HTTP status.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    pub(crate) message: String,
    pub(crate) retry_after: Option<u32>,
}

impl ApiError {
    pub(crate) fn new(
        status: StatusCode,
        code: &'static str,
        message: impl Into<String>,
    ) -> ApiError {
        ApiError {
            status,
            code,
            message: message.into(),
            retry_after: None,
        }
    }

    pub(crate) fn bad_input(message: impl Into<String>) -> ApiError {
        ApiError::new(StatusCode::BAD_REQUEST, "bad_input", message)
    }
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: ErrorDetail<'a>,
}

#[derive(Serialize)]
pub(crate) struct ErrorDetail<'a> {
    code: &'a str,
    message: &'a str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            error: ErrorDetail {
                code: self.code,
                message: &self.message,
            },
        };
        let mut response = (self.status, Json(body)).into_response();
        if let Some(seconds) = self.retry_after {
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from(seconds));
        }
        response
    }
}

/// A body already written as JSON text, sent as `application/json`.
pub(crate) struct JsonText(pub(crate) String);

impl IntoResponse for JsonText {
    fn into_response(self) -> Response {
        (
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            )],
            self.0,
        )
            .into_response()
    }
}

pub(crate) type ApiResult<T> = Result<Json<T>, ApiError>;

pub(crate) async fn method_not_allowed(method: Method) -> ApiError {
    ApiError::new(
        StatusCode::METHOD_NOT_ALLOWED,
        "method_not_allowed",
        format!("{method} is not allowed on this endpoint"),
    )
}

pub(crate) async fn not_found() -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, "not_found", "no such endpoint")
}
