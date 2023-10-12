use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Reservation {
    pub user_id: String,
    pub place_id: String,
    pub reservation_timestamp: OffsetDateTime,
}