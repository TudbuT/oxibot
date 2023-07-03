use crate::{Data, Error, EMBED_COLOR};
use poise::serenity_prelude::{ChannelId, Context, GuildId, Message, MessageId, Reaction, User};
use sqlx::Error as SQLxError;

pub async fn add_starboard_tables(
    data: &Data,
    guild_id: GuildId,
    channel_id: ChannelId,
    emoji: &str,
    min_reactions: i32,
) -> Result<(), SQLxError> {
    sqlx::query!(
        "INSERT INTO starboard (guild_id, emoji, starboard_channel, min_reactions) VALUES ($1, $2, $3, $4)",
        &guild_id.as_u64().to_be_bytes(),
        emoji,
        &channel_id.as_u64().to_be_bytes(),
        min_reactions
    )
    .execute(&data.db)
    .await?;

    Ok(())
}

/// Manages the starboard response to a change in reactions
pub async fn manage_starboard_entry(
    ctx: &Context,
    data: &Data,
    reaction: &Reaction,
) -> Result<(), Error> {
    // Check if this reaction is in a guild, and get guild id
    let guild_id = match reaction.guild_id {
        Some(guild) => guild.as_u64().to_be_bytes(),
        None => return Ok(()),
    };

    let emoji = reaction.emoji.clone();
    let emoji_string = emoji.to_string();

    let possible_starboard = sqlx::query!(
        r#"SELECT starboard_channel as "starboard_channel: [u8; 8]", min_reactions FROM starboard 
                    WHERE starboard.guild_id = $1 AND starboard.emoji = $2"#,
        &guild_id,
        emoji_string
    )
    .fetch_optional(&data.db)
    .await?
    .map(|record| (record.starboard_channel, record.min_reactions));

    // Return if we don't have a starboard for this emoji
    let (starboard_channel, min_reactions) = match possible_starboard {
        Some(starboard) => starboard,
        None => return Ok(()),
    };

    let starboard_channel = ChannelId(u64::from_be_bytes(starboard_channel));

    let message = reaction.message(ctx).await?;

    let reactions = message
        .reaction_users(ctx, emoji, Some(100), None)
        .await
        .unwrap_or(vec![]);

    let length = {
        if reactions.contains(&message.author) {
            reactions.len() - 1
        } else {
            reactions.len()
        }
    };

    if length >= min_reactions.try_into().unwrap() {
        add_or_edit_starboard_entry(
            ctx,
            data,
            &message,
            &reactions,
            emoji_string.as_str(),
            starboard_channel,
        )
        .await?;
    } else {
        remove_starboard_entry_with_channel(ctx, data, &message.id, starboard_channel).await?;
    }

    Ok(())
}

/// Edits a starboard entry, or creates one if one does not exist
async fn add_or_edit_starboard_entry(
    ctx: &Context,
    data: &Data,
    message: &Message,
    reactions: &[User],
    emoji_string: &str,
    channel: ChannelId,
) -> Result<(), Error> {
    let possible_entry = sqlx::query!(
        r#"SELECT starboard_channel as "starboard_channel: [u8; 8]", starboard_post_id as "starboard_post_id: [u8; 8]", reaction_count FROM starboard_tracked 
                    WHERE starboard_tracked.message_id = $1 AND starboard_tracked.emoji = $2"#,
        &message.id.as_u64().to_be_bytes(),
        emoji_string
    )
    .fetch_optional(&data.db)
    .await?
    .map(|record| (ChannelId(u64::from_be_bytes(record.starboard_channel)), MessageId(u64::from_be_bytes(record.starboard_post_id)), record.reaction_count));

    match possible_entry {
        Some((channel, post_id, reactions)) => {
            edit_starboard_entry(ctx, data, post_id, channel, reactions, emoji_string).await?
        }
        None => {
            add_starboard_entry(
                ctx,
                data,
                message,
                channel,
                emoji_string,
                reactions.len().try_into()?,
            )
            .await?
        }
    }

    Ok(())
}

/// Creates a new starboard entry
async fn add_starboard_entry(
    ctx: &Context,
    data: &Data,
    message: &Message,
    starboard_channel: ChannelId,
    emoji_string: &str,
    current_reactions: i32,
) -> Result<(), Error> {
    // Formatting message
    let link = format!("[Jump!]({})", message.link());
    let channel = message.channel(ctx).await?.to_string();

    let starboard_message = format!("{channel} | {emoji_string} {current_reactions}");

    // Post embed
    let post = starboard_channel
        .send_message(ctx, |m| {
            m.content(starboard_message).embed(|e| {
                e.author(|a| {
                    a.icon_url(message.author.face())
                        .name(message.author.name.clone())
                })
                .description(message.content_safe(ctx))
                .field("Source", link, false)
                .color(EMBED_COLOR)
                .footer(|f| f.text(message.id))
                .timestamp(message.timestamp.to_string());

                // if the message has a file, try to make it a thumbnail
                if !message.attachments.is_empty() {
                    e.image(message.attachments[0].url.clone())
                } else {
                    e
                }
            })
        })
        .await?;

    // Add entry
    sqlx::query!(
        r#"INSERT INTO starboard_tracked 
        (message_id, emoji, starboard_channel, starboard_post_id, reaction_count) VALUES ($1, $2, $3, $4, $5)"#,
        &message.id.as_u64().to_be_bytes(),
        emoji_string,
        &starboard_channel.as_u64().to_be_bytes(),
        &post.id.as_u64().to_be_bytes(),
        current_reactions
    ).execute(&data.db)
    .await?;

    Ok(())
}

/// Edits an existing starboard entry
async fn edit_starboard_entry(
    ctx: &Context,
    data: &Data,
    message: MessageId,
    channel: ChannelId,
    reactions: i32,
    emoji_string: &str,
) -> Result<(), Error> {
    let mut post = channel.message(ctx, message).await?;

    sqlx::query!(
        "UPDATE starboard_tracked SET reaction_count = $3 WHERE starboard_tracked.starboard_post_id = $1 AND starboard_tracked.emoji = $2",
        &message.as_u64().to_be_bytes(),
        emoji_string,
        &reactions
    ).execute(&data.db)
    .await?;

    let content =
        post.content.trim_end_matches(char::is_numeric).to_string() + &reactions.to_string();

    post.edit(ctx, |x| x.content(content)).await?;

    Ok(())
}

/// Removes a starboard entry and associated message in provided channel. Fails silently if entry does not exist.
pub async fn remove_starboard_entry_with_channel(
    ctx: &Context,
    data: &Data,
    message: &MessageId,
    starboard_channel: ChannelId,
) -> Result<(), Error> {
    let entries = sqlx::query!(
        r#"DELETE FROM starboard_tracked WHERE starboard_tracked.message_id = $1 AND starboard_tracked.starboard_channel = $2
        RETURNING starboard_post_id as "starboard_post_id: [u8; 8]""#,
        &message.as_u64().to_be_bytes(),
        &starboard_channel.as_u64().to_be_bytes()
    )
    .fetch_all(&data.db)
    .await?;

    for entry in entries {
        let message = MessageId(u64::from_be_bytes(entry.starboard_post_id));

        starboard_channel.delete_message(ctx, message).await?;
    }

    Ok(())
}

/// Removes a starboard entry and associated message. Fails silently if entry does not exist.
pub async fn remove_starboard_entry(
    ctx: &Context,
    data: &Data,
    message: &MessageId,
) -> Result<(), Error> {
    // Remove + get all entries with the message id. This should return a vec of length zero or one, but is not guaranteed
    let entries: Vec<_> = sqlx::query!(
        r#"DELETE FROM starboard_tracked WHERE starboard_tracked.message_id = $1
        RETURNING starboard_post_id as "starboard_post_id: [u8; 8]", starboard_channel as "starboard_channel: [u8; 8]""#,
        &message.as_u64().to_be_bytes(),
    )
    .fetch_all(&data.db)
    .await?;

    // If there are duplicate entries, delete all of them
    for entry in entries {
        let message = MessageId(u64::from_be_bytes(entry.starboard_post_id));

        let starboard_channel = ChannelId(u64::from_be_bytes(entry.starboard_channel));

        starboard_channel.delete_message(ctx, message).await?;
    }

    Ok(())
}

/// Remove the starboard tables associated with `channel_id`
pub async fn delete_starboard_tables(data: &Data, channel_id: &u64) -> Result<(), SQLxError> {
    let id = channel_id.to_be_bytes();

    let mut trans = data.db.begin().await?;

    sqlx::query!(
        "DELETE FROM starboard_tracked WHERE starboard_tracked.starboard_channel = $1",
        &id
    )
    .execute(&mut trans)
    .await?;

    sqlx::query!(
        "DELETE FROM starboard WHERE starboard.starboard_channel = $1",
        &id
    )
    .execute(&mut trans)
    .await?;

    trans.commit().await?;

    Ok(())
}
