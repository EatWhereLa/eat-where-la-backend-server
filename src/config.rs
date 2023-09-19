use clap::Parser;

#[derive(Parser, Clone)]
pub struct Config {
    #[clap(env, long)]
    pub environment: String,
}