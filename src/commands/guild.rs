use crate::{database::init_guild, Context, Error};

#[poise::command(prefix_command, slash_command, subcommands("init"))]
pub async fn guild(_ctx: Context<'_>, _arg: String) -> Result<(), Error> {
    Ok(())
}

/// Initializes the guild for over commands
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn init(ctx: Context<'_>) -> Result<(), Error> {
    // Since this command is guild_only this should NEVER fail
    let guild = ctx.guild_id().unwrap();

    init_guild(ctx.data(), guild).await?;

    ctx.say("Done!").await?;

    Ok(())
}
