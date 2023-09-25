use anyhow::anyhow;
// places api -> fuzzy search for restaurants based on the restaurant name ->
// search on geolocation as well(lat, long)
use bb8_postgres::bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use bb8_postgres::tokio_postgres::NoTls;
use tracing::warn;
use crate::models::restaurant::Restaurant;

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
}