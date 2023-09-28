use serde::{Serialize, Deserialize};
use std::sync::Arc;
use axum::{Extension, Json, Router};
use axum::routing::{get, post, delete};
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use tracing::warn;
use crate::controller::AppState;
use crate::repositories::postgres_repo::PostgresConnectionRepo;

pub fn router(app_state: AppState) -> Router {
    let postgres_repo = Arc::new(PostgresConnectionRepo::new(
        app_state.postgres_connection
    ));

    Router::new()
        .route("/", post(bookmark_restaurant))
        .route("/remove", delete(remove_bookmark))
        .route("/restaurants", get(retrieve_favourite_restaurants))
        .route_layer(Extension(postgres_repo))
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BookmarkRestaurant {
    pub user_id: String,
    pub place_id: String,
}

pub async fn bookmark_restaurant(
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Json(body): Json<BookmarkRestaurant>,
) -> impl IntoResponse {
    let add_to_bookmark_res = postgres_repo
        .bookmark_place(
            &body.user_id,
            &body.place_id,
        ).await;
    return match add_to_bookmark_res {
        Ok(_) => {
            (StatusCode::OK, "Successfully bookmarked restaurant").into_response()
        }
        Err(e) => {
            warn!("Something went wrong adding restaurant to bookmark due to: {}", e);
            (StatusCode::BAD_REQUEST, "Failed to add bookmark, please try again").into_response()
        }
    };
}

pub async fn remove_bookmark(
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Query(query): Query<BookmarkRestaurant>,
) -> impl IntoResponse {
    let remove_bookmark_res = postgres_repo
        .remove_bookmark(
            &query.user_id,
            &query.place_id,
        ).await;

    return match remove_bookmark_res {
        Ok(_) => {
            (StatusCode::OK, "Successfully removed bookmarked restaurant").into_response()
        }
        Err(e) => {
            warn!("Something went wrong removing restaurant from bookmarks due to: {}", e);
            (StatusCode::BAD_REQUEST, "Failed to remove bookmark, please try again").into_response()
        }
    };
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetFavouriteRestaurantParam {
    pub user_id: String,
}

pub async fn retrieve_favourite_restaurants(
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Query(query): Query<GetFavouriteRestaurantParam>,
) -> impl IntoResponse {
    let favourite_restaurants_res = postgres_repo
        .retrieve_bookmarked_places(
            &query.user_id
        )
        .await;

    return match favourite_restaurants_res {
        Ok(restaurants) => {
            (
                StatusCode::OK,
                json!(&restaurants).to_string(),
            ).into_response()
        }
        Err(e) => {
            warn!("Something went wrong retrieving favourite restaurants due to: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                "Failed to retrieve favourite restaurants, please try again!",
            ).into_response();
        }
    };
}