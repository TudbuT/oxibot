use crate::{serenity, Data, Error};
use poise::Event;
use serenity::Context;

mod channel_delete;
mod guild_member_addition;
mod message_delete;
mod reaction_add;
mod reaction_remove;
mod starboard_handler;

pub async fn event_handler(ctx: &Context, event: &Event<'_>, data: &Data) -> Result<(), Error> {
    match event {
        Event::ReactionAdd { add_reaction } => {
            reaction_add::handle(add_reaction, data, ctx).await?;
        }
        Event::ReactionRemove { removed_reaction } => {
            reaction_remove::handle(removed_reaction, data, ctx).await?
        }
        Event::MessageDelete {
            deleted_message_id, ..
        } => {
            message_delete::handle(deleted_message_id, data, ctx).await?;
        }
        Event::GuildMemberAddition { new_member } => {
            guild_member_addition::handle(new_member, data, ctx).await?;
        }
        Event::ChannelDelete { channel } => {
            channel_delete::handle(channel, data).await?;
        }
        _ => (),
    }

    Ok(())
}
