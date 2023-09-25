use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Restaurant {
    pub place_id: String,
    pub name: String,
    pub photos: Vec<Photo>,
    pub rating: f64,
    pub vicinity: String,
    pub geometry: Location,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Photo {
    pub height: i64,
    pub html_attributes: Vec<String>,
    pub photo_reference: String,
    pub width: i64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Location {
    pub lat: f64,
    pub lng: f64,
}