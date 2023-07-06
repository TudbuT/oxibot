use crate::database::starboard::{add_starboard_tables, delete_starboard_tables};
use crate::serenity::{Channel, ReactionType};
use crate::{Context, Error};

//TODO List all of the current starboards under `starboard` and `starboard list`
#[poise::command(slash_command, subcommands("create", "delete"))]
pub async fn starboard(_ctx: Context<'_>, _arg: String) -> Result<(), Error> {
    Ok(())
}

/// Creates a new starboard with a optional custom emoji
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn create(
    ctx: Context<'_>,
    #[description = "The channel to put starboard in"] starboard: Option<Channel>,
    #[description = "A custom emoji instead of a star"] emoji: Option<ReactionType>,
    #[description = "How many reactions you need to get onto starboard"] min_reactions: Option<i32>,
) -> Result<(), Error> {
    // Since this command is guild_only this should NEVER fail
    let guild = ctx.guild_id().unwrap();
    let starboard = starboard
        .as_ref()
        .map(Channel::id)
        .unwrap_or(ctx.channel_id());

    let min_reactions = min_reactions.unwrap_or(3);

    if min_reactions <= 0 {
        ctx.say("Minimum reactions should be not zero or negative!")
            .await?;
        return Ok(());
    }

    let emoji = emoji
        .map(|x| x.to_string())
        .unwrap_or_else(|| "⭐".to_string());

    add_starboard_tables(ctx.data(), guild, starboard, emoji.as_str(), min_reactions).await?;

    ctx.say("Done!").await?;

    Ok(())
}

/// Delete the starboard in the channel, if it exists
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "The channel to remove starboard from"] starboard: Option<Channel>,
    #[description = "Whether to delete the channel afterwards. Default is false."] delete: Option<
        bool,
    >,
) -> Result<(), Error> {
    let data = ctx.data();
    let starboard = starboard
        .as_ref()
        .map(Channel::id)
        .unwrap_or(ctx.channel_id());

    delete_starboard_tables(data, starboard.as_u64()).await?;

    if delete.unwrap_or(false) {
        starboard.delete(ctx).await?;
    }

    ctx.say("Done!").await?;

    Ok(())
}
