use std::sync::Arc;
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::{Extension, Router};
use axum::http::StatusCode;
use axum::routing::get;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::warn;
use crate::controller::AppState;
use crate::models::restaurant::{Location, Photo, Restaurant};
use crate::models::restaurant_image::RestaurantImage;
use crate::repositories::postgres_repo::PostgresConnectionRepo;

pub fn router(app_state: AppState) -> Router {
    let postgres_repo = Arc::new(PostgresConnectionRepo::new(
        app_state.postgres_connection.clone()
    ));

    Router::new()
        .route("/", get(proxy_google_places_api))
        .route("/photo", get(proxy_google_places_photo))
        .route("/place-details", get(proxy_google_places_details))
        .route_layer(Extension(app_state))
        .route_layer(Extension(postgres_repo))
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GooglePlacesApiParams {
    pub location: String,
    pub radius: String,
    pub r#type: String,
    pub minprice: String,
}

pub async fn proxy_google_places_api(
    Extension(app_state): Extension<AppState>,
    Extension(postgres_repo): Extension<Arc<PostgresConnectionRepo>>,
    Query(query): Query<GooglePlacesApiParams>,
) -> impl IntoResponse {
    let url = format!(
        "{}?location={}&radius={}&type={}&minprice={}&key={}",
        app_state.config.google_maps_api_url,
        query.location.replace("%2C", ","),
        query.radius,
        query.r#type,
        query.minprice,
        app_state.config.google_api_key
    );
    let google_places_api_response = app_state
        .http_client
        .get(url)
        .send()
        .await;

    let mut list_of_restaurants: Vec<Restaurant> = Vec::new();
    match google_places_api_response {
        Ok(response) => {
            // extract the restaurants here to return to frontend
            match response.json::<Value>().await {
                Ok(response_body) => {
                    for restaurant_val in response_body["results"].as_array().unwrap() {
                        let photo = restaurant_val["photos"][0].as_object().unwrap();
                        let restaurant = Restaurant {
                            place_id: restaurant_val["place_id"].to_string().replace('"', ""),
                            name: restaurant_val["name"].to_string().replace('"', "").replace("'", ""),
                            photos: Photo {
                                height: photo.get("height").unwrap().to_string().replace('"', "").parse().unwrap(),
                                photo_reference: photo.get("photo_reference").unwrap().to_string().replace('"', ""),
                                width: photo.get("width").unwrap().to_string().replace('"', "").parse().unwrap(),
                            },
                            rating: restaurant_val["rating"].to_string().replace('"', "").parse().unwrap(),
                            vicinity: restaurant_val["vicinity"].to_string().replace('"', ""),
                            geometry: Location {
                                lat: restaurant_val["geometry"]["location"]["lat"].to_string().replace('"', "").parse().unwrap(),
                                lng: restaurant_val["geometry"]["location"]["lng"].to_string().replace('"', "").parse().unwrap(),
                            },
                        };

                        list_of_restaurants.push(restaurant);
                    }
                }
                Err(e) => {
                    warn!("Failed to extract response body due to: {}", e);
                }
            }
        }
        Err(e) => {
            warn!("Failed query google places api due to: {}", e);
        }
    }

    // Store the places in database for retrieval
    let store_res = postgres_repo
        .store_browsed_places(list_of_restaurants.clone())
        .await;
    match store_res {
        Ok(_) => {}
        Err(e) => {
            warn!("Something happened: {}", e);
        }
    }

    return (
        StatusCode::OK,
        json!(list_of_restaurants).to_string(),
    ).into_response();
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GooglePlacesPhotoParams {
    pub photo_reference: String,
}

pub async fn proxy_google_places_photo(
    Extension(app_state): Extension<AppState>,
    Query(query): Query<GooglePlacesPhotoParams>,
) -> impl IntoResponse {
    let url = format!(
        "https://maps.googleapis.com/maps/api/place/photo?maxwidth=400&photoreference={}&key={}",
        query.photo_reference,
        app_state.config.google_api_key
    );

    let response = app_state
        .http_client
        .get(url)
        .send()
        .await;

    match response {
        Ok(response) => {
            let response_url = response.url();
            let domain = response_url.domain().unwrap().to_string();
            let path = response_url.path().to_string();
            let result = format!("{}{}", domain, path);
            let restaurant_image = RestaurantImage {
                image_url: result,
            };

            return (
                StatusCode::OK,
                json!(restaurant_image).to_string()
            ).into_response();
        }
        Err(e) => {
            warn!("Failed query google places api for photo due to: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong! Please try again".to_string()
            ).into_response();
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlaceDetailsParam {
    pub place_id: String,
}

pub async fn proxy_google_places_details(
    Extension(app_state): Extension<AppState>,
    Query(query): Query<PlaceDetailsParam>,
) -> impl IntoResponse {
    let url = format!(
        "https://maps.googleapis.com/maps/api/place/details/json?place_id={}&key={}",
        query.place_id,
        app_state.config.google_api_key
    );

    let response = app_state
        .http_client
        .get(url)
        .send()
        .await;

    match response {
        Ok(response) => {
            match response.json::<Value>().await {
                Ok(response_body) => {
                    return (
                        StatusCode::OK,
                        response_body.to_string()
                    ).into_response();
                }
                Err(e) => {
                    warn!("Failed to extract response body due to: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Something went wrong! Please try again".to_string()
                    ).into_response();
                }
            }
        }
        Err(e) => {
            warn!("Failed to query google places api for place details due to: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong! Please try again".to_string()
            ).into_response();
        }
    }
}