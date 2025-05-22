use chessland_puzzle_generator::http::app::app;
use chessland_puzzle_generator::{common::config::Config, domain::stockfish::Stockfish};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Welcome to puzzler");

    let conf = match Config::load() {
        Ok(conf) => {
            info!("Loaded config");
            conf
        }
        Err(_) => {
            panic!("unable to get config")
        }
    };

    let stockfish = match Stockfish::try_init() {
        Ok(stockfish) => {
            info!("initialized stockfish");
            stockfish
        }
        Err(_) => {
            error!("can't initialize stockfish, aborting...");
            panic!();
        }
    };

    let app = app(&conf, stockfish);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", conf.host, conf.port))
        .await
        .unwrap();

    info!("listening on port {}", conf.port);
    info!("puzzler is up and running");
    axum::serve(listener, app).await.unwrap();
}
