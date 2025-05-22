use crate::{domain::puzzle::Puzzle, http::app::AppState};

use axum::{Json, extract::State, http::HeaderMap};
use serde_json::Value;
use tracing::info;

use crate::{common::config::Config, domain::puzzle};

use super::error::HTTPError;

pub async fn create_puzzle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Json<Puzzle>, HTTPError> {
    info!("create puzzle endpoint is invoked");
    validate_headers(&state.conf, headers)?;
    let raw_moves = extract_payload(&body)?;
    let maybe_puzzle = puzzle::generate_puzzle_by_position_analysis(
        raw_moves,
        &mut state.stockfish.lock().unwrap(),
    );

    match maybe_puzzle {
        Ok(puzzle) => {
            info!("generated and returning puzzle");
            Ok(Json(puzzle))
        }
        Err(e) => Err(HTTPError::InvalidBody(e.to_string())),
    }
}

fn validate_headers(conf: &Config, headers: HeaderMap) -> Result<(), HTTPError> {
    match headers.get("x-api-key").and_then(|v| v.to_str().ok()) {
        Some(key) if key == conf.api_key => Ok(()),
        Some(_) => Err(HTTPError::ApiKeyInvalid),
        None => Err(HTTPError::ApiKeyMissing),
    }
}

fn extract_payload(json: &Value) -> Result<&str, HTTPError> {
    json["PGN"]
        .as_str()
        .ok_or(HTTPError::InvalidBody("invalid json".to_string()))
}
