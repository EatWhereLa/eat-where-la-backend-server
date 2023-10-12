use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RestaurantRating {
    pub user_id: String,
    pub place_id: String,
    pub rating: f64,
}