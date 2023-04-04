use crate::{serenity, Data, Error};
use serenity::{ChannelId, Context, MessageId};

use super::starboard_handler::remove_starboard_entry;

pub async fn handle(deleted_message: &MessageId, data: &Data, ctx: &Context) -> Result<(), Error> {

    

    // remove_starboard_entry(ctx, data, deleted_message, starboard_channel).await?;

    Ok(())
}
