use std::sync::Arc;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use crate::controller::AppState;

#[derive(Clone, Serialize, Deserialize, Debug)]
struct GooglePlacesApiParams {
    location: String,
    radius: String,
    r#type: String,
    minprice: String,
}

pub async fn proxy_google_places_api(
    app_state: AppState,
) -> impl IntoResponse {}