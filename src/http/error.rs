use std::fmt::Display;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

pub enum HTTPError {
    ApiKeyMissing,
    ApiKeyInvalid,
    InvalidBody(String),
    ServerError(String),
}

impl Display for HTTPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HTTPError::ApiKeyMissing => write!(f, "api key is missing"),
            HTTPError::ApiKeyInvalid => {
                write!(f, "api key mismatch, probably contains invalid characters")
            }
            HTTPError::InvalidBody(e) => write!(f, "{e}"),
            HTTPError::ServerError(e) => write!(f, "{e}"),
        }
    }
}

impl IntoResponse for HTTPError {
    fn into_response(self) -> axum::response::Response {
        let body = match self {
            HTTPError::ApiKeyMissing => json!({
                "error": "api key is missing"
            })
            .to_string(),
            HTTPError::ApiKeyInvalid => json!({
                "error": "api key mismatch, probably contains invalid characters"
            })
            .to_string(),
            HTTPError::InvalidBody(e) => json!({
                "error": e
            })
            .to_string(),
            HTTPError::ServerError(e) => json!({
                "error": e
            })
            .to_string(),
        };

        (StatusCode::BAD_REQUEST, body).into_response()
    }
}
