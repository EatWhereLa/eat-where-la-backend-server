use clap::Parser;

#[derive(Parser, Clone)]
pub struct Config {
    #[clap(env, long)]
    pub environment: String,

    #[clap(env, long)]
    pub postgres_url: String,

    #[clap(env, long)]
    pub origin_urls: String,

    #[clap(env, long)]
    pub google_maps_api_url: String,

    #[clap(env, long)]
    pub google_api_key: String,
}