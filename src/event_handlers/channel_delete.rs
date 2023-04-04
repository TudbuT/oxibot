use crate::{serenity, Data, Error, commands::starboard::delete_starboard_tables};
use serenity::GuildChannel;

pub async fn handle(deleted_channel: &GuildChannel, data: &Data) -> Result<(), Error> {
    let channel_id = deleted_channel.id.as_u64().to_be_bytes();

    delete_starboard_tables(data, channel_id).await?;

    Ok(())
}
