use anyhow::anyhow;
// places api -> fuzzy search for restaurants based on the restaurant name ->
// search on geolocation as well(lat, long)
use bb8_postgres::bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use bb8_postgres::tokio_postgres::{NoTls, Row};
use time::OffsetDateTime;
use tracing::warn;
use crate::models::restaurant::{Location, Photo, Restaurant};

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
                warn!("Failed to remove bookmarked restaurant for user: {}, due to: {}", user_id, e);
            }
        }

        Ok(favourite_restaurants)
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