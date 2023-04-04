use crate::{serenity, Data, Error};
use serenity::{Context, Reaction};

use super::starboard_handler::manage_starboard_entry;

pub async fn handle(reaction: &Reaction, data: &Data, ctx: &Context) -> Result<(), Error> {
    manage_starboard_entry(ctx, data, reaction).await?;

    Ok(())
}
