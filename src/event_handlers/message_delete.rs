use crate::{serenity, Data, Error};
use serenity::{Context, MessageId};

pub async fn handle(
    _deleted_message: &MessageId,
    _data: &Data,
    _ctx: &Context,
) -> Result<(), Error> {
    // remove_starboard_entry(ctx, data, deleted_message, starboard_channel).await?;

    Ok(())
}
