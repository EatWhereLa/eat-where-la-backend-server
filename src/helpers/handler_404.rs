use axum::http::StatusCode;
use axum::response::IntoResponse;

pub async fn page_not_found_handler() -> impl IntoResponse {
    (StatusCode::IM_A_TEAPOT, "Oops looks like you landed at the wrong endpoint, teapot")
}