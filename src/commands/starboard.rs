use crate::serenity::{Channel, ReactionType};
use crate::{Context, Error};

//TODO List all of the current starboards under `starboard` and `starboard list`
#[poise::command(prefix_command, slash_command, subcommands("create"))]
pub async fn starboard(_ctx: Context<'_>, _arg: String) -> Result<(), Error> {
    Ok(())
}

/// Creates a new starboard with a optional custom emoji
#[poise::command(
    slash_command,
    prefix_command,
    track_edits,
    guild_only,
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn create(
    ctx: Context<'_>,
    channel: Channel,
    emoji: Option<ReactionType>,
) -> Result<(), Error> {
    // Since this command is guild_only this should NEVER fail
    let guild = ctx.guild().unwrap().id.as_u64().to_be_bytes();

    let emoji = emoji
        .map(|x| x.to_string())
        .unwrap_or_else(|| "⭐".to_string());

    let starboard = channel.id().as_u64().to_be_bytes();

    sqlx::query!(
        "INSERT INTO starboard (guild_id, emoji, starboard_channel) VALUES ($1, $2, $3)",
        &guild,
        emoji,
        &starboard
    )
    .execute(&ctx.data().db)
    .await?;

    ctx.say("Done!").await?;

    Ok(())
}
