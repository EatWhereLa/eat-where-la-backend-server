use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Reservation {
    pub user_id: String,
    pub place_id: String,
    pub reservation_timestamp: i64,
    pub reservation_pax: u32,
}