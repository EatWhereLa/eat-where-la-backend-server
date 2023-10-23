use std::sync::Arc;
use axum::{Extension, Json, Router};
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post, delete, put};
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
        .route("/user", get(retrieve_user_reviews))
        .route("/restaurant", get(retrieve_restaurant_reviews))
        .route("/", post(add_review))
        .route("/", delete(remove_review))
        .route("/", put(update_review))
        .route_layer(Extension(postgres_repo))
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Review {
    pub user_id: String,
    pub place_id: String,
    pub rating: f64,
}

pub async fn add_review(
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Json(body): Json<Review>,
) -> impl IntoResponse {
    let add_user_review_res = postgres_repo
        .add_user_review(
            &body.user_id,
            &body.place_id,
            body.rating,
        ).await;

    return match add_user_review_res {
        Ok(_) => {
            (StatusCode::OK, "Successfully added review for the restaurant").into_response()
        }
        Err(e) => {
            warn!("Something went wrong adding review for restaurant due to: {}", e);
            (StatusCode::BAD_REQUEST, "Failed to add review for restaurant, please try again").into_response()
        }
    };
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RemoveReviewQuery {
    pub user_id: String,
    pub place_id: String,
}

pub async fn remove_review(
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Query(query): Query<RemoveReviewQuery>,
) -> impl IntoResponse {
    let remove_review_res = postgres_repo
        .remove_review(
            &query.user_id,
            &query.place_id,
        ).await;

    return match remove_review_res {
        Ok(_) => {
            (StatusCode::OK, "Successfully removed review for restaurant").into_response()
        }
        Err(e) => {
            warn!("Something went wrong removing review due to: {}", e);
            (StatusCode::BAD_REQUEST, "Failed to remove review, please try again.").into_response()
        }
    };
}

pub async fn update_review(
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Json(body): Json<Review>,
) -> impl IntoResponse {
    let update_review_res = postgres_repo
        .update_review(
            &body.user_id,
            &body.place_id,
            body.rating,
        ).await;

    return match update_review_res {
        Ok(_) => {
            (StatusCode::OK, "Successfully updated review for restaurant").into_response()
        }
        Err(e) => {
            warn!("Something went wrong updating review for restaurant due to: {}", e);
            (StatusCode::BAD_REQUEST, "Failed to update review for restaurant, please try again.").into_response()
        }
    };
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RetrieveRestaurantReviews {
    pub place_id: String,
}

pub async fn retrieve_restaurant_reviews(
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Query(query): Query<RetrieveRestaurantReviews>,
) -> impl IntoResponse {
    let retrieve_user_review_res = postgres_repo
        .retrieve_restaurant_reviews(
            &query.place_id
        ).await;

    return match retrieve_user_review_res {
        Ok(reviews) => {
            (
                StatusCode::OK,
                json!(&reviews).to_string()
            ).into_response()
        }
        Err(e) => {
            warn!("Something went wrong retrieving reviews for restaurant due to: {}", e);
            (StatusCode::BAD_REQUEST, "Failed to retrieve reviews for restaurant, please try again").into_response()
        }
    };
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RetrieveUserReviewsQuery {
    pub user_id: String,
}

pub async fn retrieve_user_reviews(
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Query(query): Query<RetrieveUserReviewsQuery>,
) -> impl IntoResponse {
    let user_reviewed_restaurants_res = postgres_repo
        .get_user_reviews(
            &query.user_id
        ).await;

    return match user_reviewed_restaurants_res {
        Ok(reviews) => {
            (
                StatusCode::OK,
                json!(&reviews).to_string()
            ).into_response()
        }
        Err(e) => {
            warn!("Something went wrong retrieving user reviews for restaurant due to: {}", e);
            (StatusCode::BAD_REQUEST, "Failed to retrieve user reviews for restaurant, please try again").into_response()
        }
    };
}