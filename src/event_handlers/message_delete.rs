use crate::{database::starboard::remove_starboard_entry, serenity, Data, Error};
use serenity::{Context, MessageId};

pub async fn handle(deleted_message: &MessageId, data: &Data, ctx: &Context) -> Result<(), Error> {
    remove_starboard_entry(ctx, data, deleted_message).await?;

    Ok(())
}
