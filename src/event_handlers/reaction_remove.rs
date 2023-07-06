use crate::{database::starboard::manage_starboard_entry, serenity, Data, Error};
use serenity::{Context, Reaction};

pub async fn handle(reaction: &Reaction, data: &Data, ctx: &Context) -> Result<(), Error> {
    manage_starboard_entry(ctx, data, reaction).await?;

    Ok(())
}
