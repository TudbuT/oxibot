use poise::serenity_prelude::GuildId;
use sqlx::{postgres::PgPoolOptions, Error, PgPool};
use std::env;

mod ids;
pub mod starboard;

pub use ids::*;

// Data shared across commands and events
pub struct Data {
    pub db: PgPool,
}

impl Data {
    pub fn new(database: PgPool) -> Data {
        Data { db: database }
    }
}

pub async fn init_data() -> Data {
    let database_url =
        env::var("DATABASE_URL").expect("No database url found in environment variables!");

    let database = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to database!");

    sqlx::migrate!()
        .run(&database)
        .await
        .expect("Unable to apply migrations!");

    Data::new(database)
}

/// Creates a table for the provided guild ID. Errors if there is already a table present
/// or if the database errors.
pub async fn init_guild(data: &Data, guild_id: GuildId) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO guild (discord_id) VALUES ($1)",
        &guild_id.as_u64().to_be_bytes()
    )
    .execute(&data.db)
    .await?;

    Ok(())
}
