use anyhow::anyhow;
use bb8_postgres::bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use bb8_postgres::tokio_postgres::{NoTls, Row};
use serde_json::Value;
use time::{OffsetDateTime, Time};
use tracing::warn;
use crate::models::rating::RestaurantRating;
use crate::models::reservation::Reservation;
use crate::models::restaurant::{Location, Photo, Restaurant};
use crate::models::vote::VoteHistory;

pub const RETRY_LIMIT: usize = 5;

pub struct PostgresConnectionRepo {
    postgres_connection: Pool<PostgresConnectionManager<NoTls>>,
}

impl PostgresConnectionRepo {
    pub fn new(
        postgres_connection: Pool<PostgresConnectionManager<NoTls>>,
    ) -> Self {
        Self {
            postgres_connection
        }
    }

    async fn get_postgres_connection(
        &self,
    ) -> anyhow::Result<PooledConnection<PostgresConnectionManager<NoTls>>> {
        for _ in 0..RETRY_LIMIT {
            match self.postgres_connection.get().await {
                Ok(conn) => return Ok(conn),
                Err(e) => {
                    warn!("Failed to retrieve postgres connection due to: {}, retrying in 3s", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    continue;
                }
            }
        }

        return Err(anyhow!("Failed to retrieve a valid connection from postgres pool, BAILING"));
    }

    pub async fn store_browsed_places(
        &self,
        list_of_restaurants: Vec<Restaurant>,
    ) -> anyhow::Result<()> {
        let conn = self.get_postgres_connection().await?;
        let mut stmt = String::from("INSERT INTO places \
            (place_id, name, photo_height, photo_width, photo_reference, rating, vicinity, lat, lng) \
            VALUES ");

        for restaurant in list_of_restaurants {
            let item = format!(
                "('{}', '{}', {}, {}, '{}', {}, '{}', {}, {}),",
                restaurant.place_id,
                restaurant.name,
                restaurant.photos.height,
                restaurant.photos.width,
                restaurant.photos.photo_reference,
                restaurant.rating,
                restaurant.vicinity,
                restaurant.geometry.lat,
                restaurant.geometry.lng,
            );
            stmt.push_str(&item);
        }
        stmt.remove(stmt.len() - 1);
        stmt.push_str(" ON CONFLICT DO NOTHING;");

        let res = conn
            .execute(&stmt, &[])
            .await;
        match res {
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to insert places into table due to: {}", e);
            }
        }
        Ok(())
    }

    pub async fn retrieve_restaurant(
        &self,
        place_id: &String,
    ) -> anyhow::Result<Option<Restaurant>> {
        let conn = self.get_postgres_connection().await?;
        let stmt = format!(
            "SELECT * FROM places where place_id = '{}' limit 1;",
            place_id
        );

        let res = conn
            .query(&stmt, &[])
            .await;

        match res {
            Ok(rows) => {
                if rows.len() > 0 {
                    for row in rows {
                        let restaurant = parse_row_into_restaurant(row);
                        return Ok(Some(restaurant));
                    }
                }
            }
            Err(e) => {
                warn!("Ran into an issue retrieving the restaurant with id: {}, due to: {}", place_id, e);
            }
        }
        Ok(None)
    }

    pub async fn search_for_restaurants(
        &self,
        restaurant_name: &String,
    ) -> anyhow::Result<Vec<Restaurant>> {
        let conn = self.get_postgres_connection().await?;
        let stmt = format!(
            "SELECT * FROM places WHERE name ILIKE '%{}%'",
            restaurant_name
        );

        let res = conn
            .query(&stmt, &[])
            .await;

        let mut restaurants = Vec::new();
        match res {
            Ok(rows) => {
                for row in rows {
                    let restaurant = parse_row_into_restaurant(row);

                    restaurants.push(restaurant);
                }
            }
            Err(e) => {
                warn!("Ran into an error retrieving restaurants due to: {}", e);
            }
        }

        Ok(restaurants)
    }

    pub async fn bookmark_place(
        &self,
        user_id: &String,
        place_id: &String,
    ) -> anyhow::Result<()> {
        let conn = self.get_postgres_connection().await?;
        let mut stmt = String::from("INSERT INTO user_favourite_places (user_id, place_id, timestamp) VALUES ");
        let params = format!(
            "('{}', '{}', '{}')",
            user_id,
            place_id,
            OffsetDateTime::now_utc()
        );
        stmt.push_str(&params);
        stmt.push_str(" ON CONFLICT DO NOTHING;");

        let res = conn
            .execute(&stmt, &[])
            .await;
        match res {
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to bookmark restaurant for user: {}, due to: {}", user_id, e);
            }
        }

        Ok(())
    }

    pub async fn remove_bookmark(
        &self,
        user_id: &String,
        place_id: &String,
    ) -> anyhow::Result<()> {
        let conn = self.get_postgres_connection().await?;
        let stmt = format!(
            "DELETE FROM user_favourite_places where user_id = '{}' and place_id = '{}';",
            user_id,
            place_id
        );

        let res = conn
            .execute(&stmt, &[])
            .await;
        match res {
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to remove bookmarked restaurant for user: {}, due to: {}", user_id, e);
            }
        }

        Ok(())
    }

    pub async fn retrieve_bookmarked_places(
        &self,
        user_id: &String,
    ) -> anyhow::Result<Vec<Restaurant>> {
        let conn = self.get_postgres_connection().await?;
        let stmt = format!(
            "SELECT * from places where place_id in (SELECT place_id FROM user_favourite_places where user_id = '{}');",
            user_id,
        );

        let mut favourite_restaurants: Vec<Restaurant> = Vec::new();
        let res = conn
            .query(&stmt, &[])
            .await;
        match res {
            Ok(rows) => {
                for row in rows {
                    let restaurant = parse_row_into_restaurant(row);

                    favourite_restaurants.push(restaurant);
                }
            }
            Err(e) => {
                warn!("Failed to retrieve bookmarked restaurant for user: {}, due to: {}", user_id, e);
            }
        }

        Ok(favourite_restaurants)
    }

    pub async fn add_user_review(
        &self,
        user_id: &String,
        place_id: &String,
        rating: f64,
        description: &String,
    ) -> anyhow::Result<()> {
        let conn = self.get_postgres_connection().await?;
        let mut stmt = String::from("INSERT INTO user_reviews (user_id, place_id, rating, description, timestamp) VALUES ");
        let params = format!(
            "('{}', '{}', '{}', '{}', '{}')",
            user_id,
            place_id,
            rating,
            description,
            OffsetDateTime::now_utc().unix_timestamp(),
        );
        stmt.push_str(&params);
        stmt.push_str(" ON CONFLICT DO NOTHING;");

        let res = conn
            .execute(&stmt, &[])
            .await;
        match res {
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to add review to restaurant for user: {}, due to: {}", user_id, e);
            }
        }

        Ok(())
    }

    pub async fn update_review(
        &self,
        user_id: &String,
        place_id: &String,
        rating: f64,
        description: &String,
    ) -> anyhow::Result<()> {
        let conn = self.get_postgres_connection().await?;
        let stmt = format!(
            "UPDATE user_reviews SET rating = {}, timestamp = {}, description = '{}' where user_id = '{}' and place_id = '{}';",
            rating,
            OffsetDateTime::now_utc().unix_timestamp(),
            description,
            user_id,
            place_id
        );

        let res = conn
            .execute(&stmt, &[])
            .await;
        match res {
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to update review of restaurant for user: {}, due to: {}", user_id, e);
            }
        }

        Ok(())
    }

    pub async fn remove_review(
        &self,
        user_id: &String,
        place_id: &String,
    ) -> anyhow::Result<()> {
        let conn = self.get_postgres_connection().await?;
        let stmt = format!(
            "DELETE FROM user_reviews where user_id = '{}' and place_id = '{}';",
            user_id,
            place_id
        );

        let res = conn
            .execute(&stmt, &[])
            .await;

        match res {
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to remove review on restaurant for user: {}, due to: {}", user_id, e);
            }
        }
        Ok(())
    }

    pub async fn retrieve_restaurant_reviews(
        &self,
        place_id: &String,
    ) -> anyhow::Result<Vec<RestaurantRating>> {
        let conn = self.get_postgres_connection().await?;
        let stmt = format!(
            "SELECT * from user_reviews where place_id = '{}';",
            place_id
        );

        let mut restaurant_reviews: Vec<RestaurantRating> = Vec::new();
        let res = conn.query(&stmt, &[]).await;

        match res {
            Ok(rows) => {
                for row in rows {
                    let restaurant_rating = parse_row_into_restaurant_rating(row);

                    restaurant_reviews.push(restaurant_rating);
                }
            }
            Err(e) => {
                warn!("Failed to retrieve restaurant rating for place_id: {} due to: {}", place_id, e);
            }
        }

        Ok(restaurant_reviews)
    }

    pub async fn get_user_reviews(
        &self,
        user_id: &String,
    ) -> anyhow::Result<Vec<RestaurantRating>> {
        let conn = self.get_postgres_connection().await?;
        let stmt = format!(
            "SELECT * from user_reviews where user_id = '{}';",
            user_id
        );

        let mut restaurant_reviews: Vec<RestaurantRating> = Vec::new();
        let res = conn
            .query(
                &stmt,
                &[],
            ).await;

        match res {
            Ok(rows) => {
                for row in rows {
                    let restaurant_rating = parse_row_into_restaurant_rating(row);

                    restaurant_reviews.push(restaurant_rating);
                }
            }
            Err(e) => {
                warn!("Failed to retrieve restaurant rating for user_id: {} due to: {}", user_id, e);
            }
        }

        Ok(restaurant_reviews)
    }

    pub async fn add_reservations(
        &self,
        user_id: &String,
        place_id: &String,
        reservation_timestamp: i64,
        reservation_pax: u32,
    ) -> anyhow::Result<()> {
        let conn = self.get_postgres_connection().await?;
        let mut stmt = String::from("INSERT INTO user_reservations (user_id, place_id, reservation_timestamp, reservation_pax) VALUES ");
        let params = format!(
            "('{}', '{}', {}, {})",
            user_id,
            place_id,
            reservation_timestamp,
            reservation_pax,
        );
        stmt.push_str(&params);
        stmt.push_str("ON CONFLICT DO NOTHING;");

        let res = conn
            .execute(&stmt, &[])
            .await;
        match res {
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to add reservation for user: {}, due to: {}", user_id, e);
            }
        }

        Ok(())
    }

    pub async fn remove_reservation(
        &self,
        user_id: &String,
        place_id: &String,
    ) -> anyhow::Result<()> {
        let conn = self.get_postgres_connection().await?;
        let stmt = format!(
            "DELETE FROM user_reservations where user_id = '{}' and place_id = '{}';",
            user_id,
            place_id,
        );

        let res = conn
            .execute(&stmt, &[])
            .await;

        match res {
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to remove restaurant reservation for user: {}, due to: {}", user_id, e);
            }
        }
        Ok(())
    }

    pub async fn retrieve_all_user_valid_reservations(
        &self,
        user_id: &String,
    ) -> anyhow::Result<Vec<Reservation>> {
        let conn = self.get_postgres_connection().await?;
        let stmt = format!(
            "SELECT * FROM user_reservations where user_id = '{}' and reservation_timestamp > {};",
            user_id,
            OffsetDateTime::now_utc().unix_timestamp()
        );

        let res = conn
            .query(&stmt, &[])
            .await;

        let mut reservations: Vec<Reservation> = Vec::new();
        match res {
            Ok(rows) => {
                for row in rows {
                    let user_reservation = parse_row_into_restaurant_reservation(row);

                    reservations.push(user_reservation);
                }
            }
            Err(e) => {
                warn!("Failed to retrieve user reservations for user: {}, due to: {}", user_id, e);
            }
        }
        Ok(reservations)
    }

    pub async fn retrieve_all_user_reservations(
        &self,
        user_id: &String,
    ) -> anyhow::Result<Vec<Reservation>> {
        let conn = self.get_postgres_connection().await?;
        let stmt = format!(
            "SELECT * FROM user_reservations where user_id = '{}'",
            user_id
        );

        let res = conn
            .query(&stmt, &[])
            .await;

        let mut reservations: Vec<Reservation> = Vec::new();
        match res {
            Ok(rows) => {
                for row in rows {
                    let user_reservation = parse_row_into_restaurant_reservation(row);
                    reservations.push(user_reservation);
                }
            }
            Err(e) => {
                warn!("Failed to retrieve user reservations for user: {}, due to: {}", user_id, e);
            }
        }
        Ok(reservations)
    }

    pub async fn store_vote_history(
        &self,
        user_ids: Vec<String>,
        voted_places: Value,
        vote_session_timestamp: i64,
    ) -> anyhow::Result<()> {
        let conn = self.get_postgres_connection().await?;
        let mut stmt = String::from("INSERT INTO voting_history (voted_places, vote_timestamp, voters) VALUES ");
        let mut voted_places_stmt = format!("ARRAY[");
        if let Value::Array(places) = voted_places {
            for place in places {
                voted_places_stmt.push_str(
                    &format!(
                        "'{}'::jsonb,",
                        place
                    )
                )
            }
        }
        voted_places_stmt.remove(voted_places_stmt.len() - 1);
        voted_places_stmt.push_str("]");

        let mut user_ids_stmt = format!("ARRAY[");
        for user_id in user_ids {
            user_ids_stmt.push_str(
                &format!(
                    "'{}',",
                    user_id
                )
            )
        }
        user_ids_stmt.remove(user_ids_stmt.len() - 1);
        user_ids_stmt.push_str("]");
        let params = format!(
            "({}, {}, {});",
            voted_places_stmt,
            vote_session_timestamp,
            user_ids_stmt,
        );
        stmt.push_str(&params);

        let res = conn
            .execute(&stmt, &[])
            .await;
        match res {
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to store voting history due to: {}", e);
            }
        }
        Ok(())
    }

    pub async fn retrieve_user_vote_history(
        &self,
        user_id: &String,
    ) -> anyhow::Result<Vec<VoteHistory>> {
        let conn = self.get_postgres_connection().await?;
        let stmt = format!(
            "SELECT * FROM voting_history WHERE '{}' = ANY(voters)",
            user_id
        );

        let res = conn
            .query(&stmt, &[])
            .await;

        let mut vote_histories: Vec<VoteHistory> = Vec::new();
        match res {
            Ok(rows) => {
                for row in rows {
                    let vote_history = parse_row_into_vote_history(row);

                    vote_histories.push(vote_history);
                }
            }
            Err(e) => {
                warn!("Failed to retrieve user's voting histories: {}", e);
            }
        }
        Ok(vote_histories)
    }
}

fn parse_row_into_restaurant(
    row: Row
) -> Restaurant {
    Restaurant {
        place_id: row.get("place_id"),
        name: row.get("name"),
        photos: Photo {
            height: row.get::<&str, i32>("photo_height") as i64,
            photo_reference: row.get("photo_reference"),
            width: row.get::<&str, i32>("photo_width") as i64,
        },
        rating: row.get::<&str, f64>("rating"),
        vicinity: row.get("vicinity"),
        geometry: Location {
            lat: row.get::<&str, f64>("lat"),
            lng: row.get::<&str, f64>("lng"),
        },
    }
}

fn parse_row_into_restaurant_rating(
    row: Row,
) -> RestaurantRating {
    RestaurantRating {
        user_id: row.get("user_id"),
        place_id: row.get("place_id"),
        rating: row.get::<&str, f64>("rating"),
        description: row.get("description"),
        timestamp: row.get::<&str, i32>("timestamp") as i64,
    }
}

fn parse_row_into_restaurant_reservation(
    row: Row,
) -> Reservation {
    let epoch_time = row.get::<&str, i32>("reservation_timestamp");
    let pax = row.get::<&str, i32>("reservation_pax");
    let user_id = row.get::<&str, &str>("user_id");
    let place_id = row.get::<&str, &str>("place_id");

    Reservation {
        user_id: user_id.to_string(),
        place_id: place_id.to_string(),
        reservation_timestamp: epoch_time as i64,
        reservation_pax: pax as u32,
    }
}

fn parse_row_into_vote_history(
    row: Row,
) -> VoteHistory {
    let user_ids = row.get::<&str, Vec<String>>("voters");
    let vote_timestamp = row.get::<&str, i32>("vote_timestamp");
    let voted_places = row.get::<&str, Vec<Value>>("voted_places");
    VoteHistory {
        user_ids,
        vote_timestamp: vote_timestamp as i64,
        voted_places,
    }
}