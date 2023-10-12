use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::Context;
use axum::http::HeaderValue;
use axum::{Extension, Router};
use bb8_postgres::bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use bb8_postgres::tokio_postgres::NoTls;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client, Method};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;
use crate::config::Config;
use crate::helpers::handler_404::page_not_found_handler;

pub mod bookmarks_controller;
pub mod google_places_api;
pub mod health_check;
pub mod restaurant_controller;
pub mod user_reservation_controller;
pub mod user_review_controller;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub postgres_connection: Pool<PostgresConnectionManager<NoTls>>,
    pub http_client: Client,
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
    let reqwest_client = Client::new();

    let app_state = AppState {
        config: Arc::new(config.clone()),
        postgres_connection,
        http_client: reqwest_client,
    };

    let application = router_endpoints(app_state.clone())
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
        .fallback(page_not_found_handler);

    let port = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("API server listening on port: {}", port);
    axum::Server::bind(&port)
        .serve(application.into_make_service())
        .await
        .context("Error spinning up the API server")
}

pub fn router_endpoints(
    app_state: AppState,
) -> Router {
    health_check::router()
        .nest("/google", google_places_api::router(app_state.clone()))
        .nest("/restaurant", restaurant_controller::router(app_state.clone()))
        .nest("/bookmark", bookmarks_controller::router(app_state.clone()))
        .nest("/review", user_review_controller::router(app_state.clone()))
        .nest("/reservation", user_reservation_controller::router(app_state.clone()))
}