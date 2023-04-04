use crate::{serenity, Data, Error, data::starboard::remove_starboard_entry};
use serenity::{Context, MessageId};

pub async fn handle(
    deleted_message: &MessageId,
    data: &Data,
    ctx: &Context,
) -> Result<(), Error> {

    remove_starboard_entry(ctx, data, deleted_message).await?;

    Ok(())
}
