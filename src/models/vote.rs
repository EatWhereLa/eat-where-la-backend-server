use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VoteHistory {
    pub user_ids: Vec<String>,
    pub vote_timestamp: i64,
    pub voted_places: Vec<Value>,
}