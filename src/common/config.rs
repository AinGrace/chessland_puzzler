use std::{env, error::Error};

use dotenvy::dotenv;

#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: String,

    pub api_key: String,
    pub chessland_endpoint: String,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn Error>> {
        dotenv()?;

        Ok(Self {
            host: env::var("HOST")?,
            port: env::var("PORT")?,
            api_key: env::var("API_KEY")?,
            chessland_endpoint: env::var("CHESSLAND_ENDPOINT")?,
        })
    }
}
