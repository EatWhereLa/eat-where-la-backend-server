use std::sync::Arc;
use axum::{Extension, Router};
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::routing::{get};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::warn;
use crate::controller::AppState;
use crate::repositories::postgres_repo::PostgresConnectionRepo;

pub fn router(app_state: AppState) -> Router {
    let postgres_repo = Arc::new(PostgresConnectionRepo::new(
        app_state.postgres_connection
    ));

    Router::new()
        .route("/", get(retrieve_restaurant))
        .route("/search", get(search_restaurants_by_name))
        .route_layer(Extension(postgres_repo))
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetRestaurantParam {
    pub place_id: String,
}

pub async fn retrieve_restaurant(
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Query(query): Query<GetRestaurantParam>,
) -> impl IntoResponse {
    let restaurant_res = postgres_repo
        .retrieve_restaurant(
            &query.place_id
        ).await;

    return match restaurant_res {
        Ok(restaurant) => {
            if restaurant.is_some() {
                (
                    StatusCode::OK,
                    json!(&restaurant.unwrap()).to_string()
                ).into_response()
            } else {
                (
                    StatusCode::OK,
                    json!("{}").to_string()
                ).into_response()
            }
        }
        Err(e) => {
            warn!("Something went wrong retrieving restaurant due to: {}", e);
            (
                StatusCode::BAD_REQUEST,
                "Failed to retrieve restaurant, please try again!"
            ).into_response()
        }
    };
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SearchRestaurantParam {
    pub restaurant_name: String,
}

pub async fn search_restaurants_by_name(
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Query(query): Query<SearchRestaurantParam>,
) -> impl IntoResponse {
    let restaurants_res = postgres_repo
        .search_for_restaurants(
            &query.restaurant_name
        ).await;

    return match restaurants_res {
        Ok(restaurants) => {
            (
                StatusCode::OK,
                json!(&restaurants).to_string()
            ).into_response()
        }
        Err(e) => {
            warn!("Something went wrong searching for restaurants due to: {}", e);
            (
                StatusCode::BAD_REQUEST,
                "{}".to_string()
            ).into_response()
        }
    };
}