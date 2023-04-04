use crate::{serenity, Data, Error, data::starboard::manage_starboard_entry};
use serenity::{Context, Reaction};

pub async fn handle(reaction: &Reaction, data: &Data, ctx: &Context) -> Result<(), Error> {
    manage_starboard_entry(ctx, data, reaction).await?;

    Ok(())
}
