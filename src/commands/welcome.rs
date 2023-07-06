use crate::serenity::Channel;
use crate::{Context, Error};

#[poise::command(prefix_command, slash_command, subcommands("message", "channel"))]
pub async fn welcome(_ctx: Context<'_>, _arg: String) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, subcommands("add", "list"))]
pub async fn message(_ctx: Context<'_>, _arg: String) -> Result<(), Error> {
    Ok(())
}

///Use `{}` to indicate where the username should be placed, otherwise it is placed at the end
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn add(ctx: Context<'_>, message: String) -> Result<(), Error> {
    // SAFETY: Since this command is guild_only this should NEVER fail
    let guild = ctx.guild_id().unwrap().0 as i64;

    sqlx::query!("UPDATE guild SET welcome_messages = array_append(welcome_messages, $1) WHERE guild.discord_id = $2", message, guild)
        .execute(&ctx.data().db)
        .await?;

    ctx.say("Done!").await?;

    Ok(())
}

///Lists all current welcome messages
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    // SAFETY: Since this command is guild_only this should NEVER fail
    let guild = ctx.guild_id().unwrap().0 as i64;

    let welcome_messages = sqlx::query!(
        "SELECT welcome_messages FROM guild WHERE guild.discord_id = $1",
        guild
    )
    .fetch_one(&ctx.data().db)
    .await?
    .welcome_messages;

    let mut formated_messages: String = welcome_messages
        .into_iter()
        .map(|message| format!("```\n{message}```\n"))
        .collect();

    formated_messages.pop();

    ctx.defer_ephemeral().await?;
    ctx.say(formated_messages).await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, subcommands("change"))]
pub async fn channel(_ctx: Context<'_>, _arg: String) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    track_edits,
    guild_only,
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn change(ctx: Context<'_>, channel: Channel) -> Result<(), Error> {
    let channel = channel.id().0 as i64;
    // SAFETY: Since this command is guild_only this should NEVER fail
    let guild = ctx.guild_id().unwrap().0 as i64;

    sqlx::query!(
        "UPDATE guild SET welcome_channel = $1 WHERE guild.discord_id = $2",
        channel,
        guild
    )
    .execute(&ctx.data().db)
    .await?;

    ctx.say("Done, make sure to have at least a single welcome message!")
        .await?;

    Ok(())
}
