use crate::{Context, Error};

#[poise::command(prefix_command, guild_only, slash_command, aliases("t", "tag"))]
pub async fn tags(ctx: Context<'_>, arg: String) -> Result<(), Error> {
    // SAFETY: Since this command is guild_only this should NEVER fail
    let guild = ctx.guild_id().unwrap().0 as i64;

    let possible_tag = sqlx::query!(
        "SELECT tag_description FROM tag WHERE tag.guild_id = $1 AND tag.command_name = $2",
        &guild,
        arg
    )
    .fetch_optional(&ctx.data().db)
    .await?;

    match possible_tag {
        Some(record) => ctx.say(record.tag_description).await?,
        None => ctx.say("Could not find tag!").await?,
    };

    Ok(())
}

#[poise::command(prefix_command, guild_only, slash_command, aliases("tag-list"))]
pub async fn tag_list(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    // SAFETY: Since this command is guild_only this should NEVER fail
    let guild = ctx.guild_id().unwrap().0 as i64;

    let tags = sqlx::query!(
        "SELECT command_name, tag_description FROM tag WHERE tag.guild_id = $1",
        guild
    )
    .fetch_all(&ctx.data().db)
    .await?;

    //TODO make this with embeds

    let mut content;

    let content = match tags.len() {
        0 => "No tags for this server have been found",
        _ => {
            content = String::new();

            for tag in tags {
                content += &format!("> {}\n{}\n\n", tag.command_name, tag.tag_description);
            }

            &content
        }
    };

    ctx.say(content).await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    aliases("tag-edit"),
    subcommands("add", "edit", "remove")
)]
pub async fn tag_edit(_ctx: Context<'_>, _arg: String) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    track_edits,
    required_permissions = "MANAGE_MESSAGES"
)]
async fn add(ctx: Context<'_>, name: String, #[rest] description: String) -> Result<(), Error> {
    // SAFETY: Since this command is guild_only this should NEVER fail
    let guild = ctx.guild_id().unwrap().0 as i64;

    sqlx::query!(
        "INSERT INTO tag (guild_id, command_name, tag_description) VALUES ($1, $2, $3)",
        guild,
        name,
        description
    )
    .execute(&ctx.data().db)
    .await?;

    ctx.say("Done!").await?;

    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    track_edits,
    required_permissions = "MANAGE_MESSAGES"
)]
async fn edit(
    ctx: Context<'_>,
    name: String,
    #[rest] new_description: String,
) -> Result<(), Error> {
    // SAFETY: Since this command is guild_only this should NEVER fail
    let guild = ctx.guild_id().unwrap().0 as i64;

    sqlx::query!(
        "UPDATE tag SET tag_description = $1 WHERE tag.guild_id = $2 AND tag.command_name = $3",
        new_description,
        guild,
        name
    )
    .execute(&ctx.data().db)
    .await?;

    ctx.say("Done!").await?;

    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    aliases("rem"),
    track_edits,
    required_permissions = "MANAGE_MESSAGES"
)]
async fn remove(ctx: Context<'_>, name: String) -> Result<(), Error> {
    // SAFETY: Since this command is guild_only this should NEVER fail
    let guild = ctx.guild_id().unwrap().0 as i64;

    sqlx::query!(
        "DELETE FROM tag WHERE tag.guild_id = $1 AND tag.command_name = $2",
        guild,
        name
    )
    .execute(&ctx.data().db)
    .await?;

    ctx.say("Done!").await?;

    Ok(())
}
