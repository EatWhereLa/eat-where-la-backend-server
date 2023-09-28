use std::sync::Arc;
use axum::{Extension, Router};
use axum::extract::Query;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use crate::controller::AppState;
use crate::repositories::postgres_repo::PostgresConnectionRepo;

pub fn router(app_state: AppState) -> Router {
    let postgres_repo = Arc::new(PostgresConnectionRepo::new(
        app_state.postgres_connection.clone()
    ));

    Router::new()
        .route("/bookmark", post(bookmark_restaurant))
        .route_layer(Extension(app_state))
        .route_layer(Extension(postgres_repo))
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BookmarkRestaurantParams {
    pub user_id: String,
    pub place_id: String,
}

pub async fn bookmark_restaurant(
    Extension(app_state): Extension<AppState>,
    Extension(postgres_repo): Extension<PostgresConnectionRepo>,
    Query(query): Query<BookmarkRestaurantParams>,
) -> impl IntoResponse {

}