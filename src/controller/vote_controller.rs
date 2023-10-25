use std::sync::Arc;
use axum::{Extension, Json, Router};
use axum::extract::Query;
use axum::response::IntoResponse;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::warn;
use axum::routing::{get, post};
use crate::controller::AppState;
use crate::repositories::postgres_repo::PostgresConnectionRepo;

pub fn router(app_state: AppState) -> Router {
    let postgres_repo = Arc::new(PostgresConnectionRepo::new(
        app_state.postgres_connection
    ));

    Router::new()
        .route("/", post(persist_vote_history))
        .route("/", get(retrieve_vote_history))
        .route_layer(Extension(postgres_repo))
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VotingHistory {
    user_ids: Vec<String>,
    voted_places: Value,
    vote_timestamp: i64,
}

pub async fn persist_vote_history(
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Json(body): Json<VotingHistory>,
) -> impl IntoResponse {
    let store_vote_history_res = postgres_repo
        .store_vote_history(
            body.user_ids,
            body.voted_places,
            body.vote_timestamp,
        ).await;

    return match store_vote_history_res {
        Ok(_) => {
            (StatusCode::OK, "Successfully persisted voting history record").into_response()
        }
        Err(e) => {
            warn!("Something went wrong persisting vote history due to: {}", e);
            (StatusCode::BAD_REQUEST, "Failed to persist vote history, please try again.").into_response()
        }
    };
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VoteHistoryParam {
    user_id: String,
}

pub async fn retrieve_vote_history(
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Query(query): Query<VoteHistoryParam>,
) -> impl IntoResponse {
    let user_vote_histories_res = postgres_repo
        .retrieve_user_vote_history(
            &query.user_id
        ).await;

    return match user_vote_histories_res {
        Ok(vote_histories) => {
            (StatusCode::OK, json!(vote_histories).to_string()).into_response()
        }
        Err(e) => {
            warn!("Something went wrong retrieving user's voting histories: {}", e);
            (StatusCode::BAD_REQUEST, "Failed to retrieve user's voting histories, please try again.").into_response()
        }
    };
}