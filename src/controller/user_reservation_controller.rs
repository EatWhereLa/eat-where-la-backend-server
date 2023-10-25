use std::sync::Arc;
use axum::{Extension, Json, Router};
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post, delete};
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
        .route("/", get(get_all_existing_reservations))
        .route("/list", get(get_all_reservations))
        .route("/", post(add_reservation))
        .route("/", delete(delete_reservation))
        .route_layer(Extension(postgres_repo))
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReserveRestaurant {
    pub user_id: String,
    pub place_id: String,
    pub reservation_time: i64,
    pub reservation_pax: u32,
}

pub async fn add_reservation(
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Json(body): Json<ReserveRestaurant>,
) -> impl IntoResponse {
    let add_reservation_res = postgres_repo
        .add_reservations(
            &body.user_id,
            &body.place_id,
            body.reservation_time,
            body.reservation_pax,
        ).await;

    return match add_reservation_res {
        Ok(_) => {
            (StatusCode::OK, "Successfully added reservation for restaurant").into_response()
        }
        Err(e) => {
            warn!("Something went wrong adding reservation for restaurant due to: {}", e);
            (StatusCode::BAD_REQUEST, "Failed to add reservation, please try again.").into_response()
        }
    };
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeleteReservationQuery {
    pub user_id: String,
    pub place_id: String,
}

pub async fn delete_reservation(
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Query(query): Query<DeleteReservationQuery>,
) -> impl IntoResponse {
    let remove_reservation_res = postgres_repo
        .remove_reservation(
            &query.user_id,
            &query.place_id,
        ).await;

    return match remove_reservation_res {
        Ok(_) => {
            (StatusCode::OK, "Successfully removed reservation").into_response()
        }
        Err(e) => {
            warn!("Something went wrong removing reservation due to: {}", e);
            (StatusCode::BAD_REQUEST, "Failed to remove reservation, please try again.").into_response()
        }
    };
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetReservationQuery {
    pub user_id: String,
}

pub async fn get_all_existing_reservations(
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Query(query): Query<GetReservationQuery>,
) -> impl IntoResponse {
    let user_reservations_res = postgres_repo
        .retrieve_all_user_valid_reservations(
            &query.user_id
        ).await;

    return match user_reservations_res {
        Ok(reservations) => {
            (StatusCode::OK, json!(reservations).to_string()).into_response()
        }
        Err(e) => {
            warn!("Something went wrong retrieving user's reservations due to: {}", e);
            (StatusCode::BAD_REQUEST, "Failed to retrieve reservations, please try again.").into_response()
        }
    };
}

pub async fn get_all_reservations(
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Query(query): Query<GetReservationQuery>,
) -> impl IntoResponse {
    let user_reservations_res = postgres_repo
        .retrieve_all_user_reservations(
            &query.user_id
        ).await;

    return match user_reservations_res {
        Ok(reservations) => {
            (StatusCode::OK, json!(reservations).to_string()).into_response()
        }
        Err(e) => {
            warn!("Something went wrong retrieving user's reservations due to: {}", e);
            (StatusCode::BAD_REQUEST, "Failed to retrieve reservations, please try again.").into_response()
        }
    };
}

