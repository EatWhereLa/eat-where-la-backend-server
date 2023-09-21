use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

pub fn router() -> Router {
    Router::new().route("/health", get(get_health_check))
}

/// Misc endpoint for individual use case
async fn get_health_check() -> impl IntoResponse
{
    (
        StatusCode::OK,
        "Server is healthy"
    ).into_response()
}
