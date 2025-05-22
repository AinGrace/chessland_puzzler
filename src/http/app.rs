use std::sync::{Arc, Mutex};

use axum::{routing::post, Router};

use crate::{common::config::Config, domain::stockfish::Stockfish};

use super::handler::create_puzzle;

#[derive(Clone)]
pub struct AppState {
    pub conf: Config,
    pub stockfish: Arc<Mutex<Stockfish>>,
}

pub fn app(conf: &Config, stockfish: Stockfish) -> Router {
    let state = AppState { conf: conf.clone(), stockfish: Arc::new(Mutex::new(stockfish)) };
    Router::new().route("/chessland/puzzler/generate", post(create_puzzle)).with_state(state)
}
