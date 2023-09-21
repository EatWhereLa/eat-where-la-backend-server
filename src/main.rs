use std::str::FromStr;
use std::time::Duration;
use bb8_postgres::bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use bb8_postgres::tokio_postgres::NoTls;
use clap::Parser;
use dotenv::dotenv;
use crate::config::Config;

pub mod controller;
pub mod helpers;
pub mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let config = Config::parse();
    tracing_subscriber::fmt::init();

    let cores = num_cpus::get();

    let postgres_config = bb8_postgres::tokio_postgres::config::Config::from_str(&config.postgres_url)?;
    let postgres_manager = PostgresConnectionManager::new(postgres_config, NoTls);

    let pool_build_result = Pool::builder()
        .connection_timeout(Duration::from_secs(15))
        .max_size((cores * 2) as u32)
        .build(postgres_manager)
        .await?;

    controller::serve(
        pool_build_result,
        &config,
    ).await?;

    Ok(())
}
