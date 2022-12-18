use crate::{serenity::Context, Data, Error};
use poise::Event;

mod guild_member_addition;
mod reaction_add;
mod reaction_remove;

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
            data.starboard_candidates
                .remove(deleted_message_id.as_u64());
            let tracked = data.starboard_tracked.remove(deleted_message_id.as_u64());

            if let Some((_, (starboard, _))) = tracked {
                starboard.delete(ctx).await?;
            }
        }
        Event::GuildMemberAddition { new_member } => {
            guild_member_addition::handle(new_member, data, ctx).await?;
        }
        _ => (),
    }

    Ok(())
}
