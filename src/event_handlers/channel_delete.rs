use crate::{database::starboard::delete_starboard_tables, serenity, Data, Error};
use serenity::GuildChannel;

pub async fn handle(deleted_channel: &GuildChannel, data: &Data) -> Result<(), Error> {
    delete_starboard_tables(data, deleted_channel.id).await?;

    Ok(())
}
