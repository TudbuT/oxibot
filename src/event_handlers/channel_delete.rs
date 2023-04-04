use crate::{data::starboard::delete_starboard_tables, serenity, Data, Error};
use serenity::GuildChannel;

pub async fn handle(deleted_channel: &GuildChannel, data: &Data) -> Result<(), Error> {
    let channel_id = deleted_channel.id.as_u64();

    delete_starboard_tables(data, channel_id).await?;

    Ok(())
}
