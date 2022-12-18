use crate::{Context, Error};

#[poise::command(prefix_command, slash_command, subcommands("init"))]
pub async fn guild(_ctx: Context<'_>, _arg: String) -> Result<(), Error> {
    Ok(())
}

/// Initializes the guild for over commands
#[poise::command(
    slash_command,
    prefix_command,
    track_edits,
    guild_only,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn init(ctx: Context<'_>) -> Result<(), Error> {
    // Since this command is guild_only this should NEVER fail
    let guild = ctx.guild().unwrap().id.as_u64().to_be_bytes();

    sqlx::query!("INSERT INTO guild (discord_id) VALUES ($1)", &guild)
        .execute(&ctx.data().db)
        .await?;

    ctx.say("Done!").await?;

    Ok(())
}
