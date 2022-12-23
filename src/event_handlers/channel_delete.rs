use crate::{serenity, Data, Error};
use serenity::GuildChannel;

pub async fn handle(deleted_message: &GuildChannel, data: &Data) -> Result<(), Error> {
    let id = deleted_message.id.as_u64().to_be_bytes();

    sqlx::query!(
        "DELETE FROM starboard_tracked WHERE starboard_tracked.starboard_channel = $1",
        &id
    )
    .execute(&data.db)
    .await?;

    sqlx::query!(
        "DELETE FROM starboard WHERE starboard.starboard_channel = $1",
        &id
    )
    .execute(&data.db)
    .await?;

    Ok(())
}
