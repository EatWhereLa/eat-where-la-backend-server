use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::Context;
use axum::handler::HandlerWithoutStateExt;
use axum::http::HeaderValue;
use axum::{Extension, Router};
use bb8_postgres::bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use bb8_postgres::tokio_postgres::NoTls;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Method;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;
use crate::config::Config;
use crate::helpers::handler_404::page_not_found_handler;

pub mod health_check;

#[derive(Clone)]
pub struct AppState {
    config: Arc<Config>,
    postgres_connection: Pool<PostgresConnectionManager<NoTls>>,
}

pub async fn serve(
    postgres_connection: Pool<PostgresConnectionManager<NoTls>>,
    config: &Config,
) -> anyhow::Result<()> {
    let origins: Vec<HeaderValue> = config
        .origin_urls
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect::<Vec<HeaderValue>>();

    let app_state = AppState {
        config: Arc::new(config.clone()),
        postgres_connection,
    };

    let application = router_endpoints()
        .layer(
            ServiceBuilder::new()
                .layer(
                    CorsLayer::new()
                        .allow_methods([
                            Method::GET,
                            Method::POST,
                            Method::PUT,
                            Method::DELETE,
                            Method::OPTIONS
                        ])
                        .allow_origin(origins)
                        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
                )
                .layer(Extension(app_state))
        )
        .fallback(page_not_found_handler.into_service());

    let port = SocketAddr::from([127, 0, 0, 0], 5000);
    info!("API server listening on port: {}", port);
    axum::Server::bind(&port)
        .serve(application.into_make_service())
        .await
        .context("Error spinning up the API server")
}

pub fn router_endpoints() -> Router {
    health_check::router()
}