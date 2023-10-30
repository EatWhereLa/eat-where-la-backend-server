use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RestaurantRating {
    pub user_id: String,
    pub place_id: String,
    pub rating: f64,
    pub description: String,
    pub timestamp: i64,
}