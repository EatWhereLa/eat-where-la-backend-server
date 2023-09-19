use clap::Parser;
use dotenv::dotenv;
use crate::config::Config;

pub mod config;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = Config::parse();

}
