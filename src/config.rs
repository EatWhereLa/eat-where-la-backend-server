use clap::Parser;

#[derive(Parser, Clone)]
pub struct Config {
    #[clap(env, long)]
    pub environment: String,

    #[clap(env, long)]
    pub postgres_url: String,
}