use poise::serenity_prelude::{ChannelId, Context, Message, MessageId, Reaction, User};

use crate::{Data, Error, EMBED_COLOR};

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
    let starboard: ([u8; 8], i32) = match possible_starboard {
        Some((starboard_channel, min_reactions)) => (starboard_channel, min_reactions),
        None => return Ok(()),
    };

    let message = reaction.message(ctx).await?;

    let reactions = message
        .reaction_users(ctx, emoji, None, None)
        .await
        .unwrap_or(vec![]);

    let starboard_channel = ChannelId(u64::from_be_bytes(starboard.0));
    let min_reactions = starboard.1;

    let mut length: i32 = reactions.len().try_into()?;

    if reactions.contains(&message.author) {
        length -= 1;
    }

    if length >= min_reactions {
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
        remove_starboard_entry(ctx, data, &message.id, starboard_channel).await?;
    }

    Ok(())
}

async fn add_or_edit_starboard_entry(
    ctx: &Context,
    data: &Data,
    message: &Message,
    reactions: &Vec<User>,
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

/// Removes a starboard entry and associated message. Fails silently if entry does not exist.
pub async fn remove_starboard_entry(
    ctx: &Context,
    data: &Data,
    message: &MessageId,
    starboard_channel: ChannelId,
) -> Result<(), Error> {
    // Remove + get all entries with the message id. This should return a vec of length zero or one, but is not guaranteed
    let entries: Vec<_> = sqlx::query!(
        r#"DELETE FROM starboard_tracked WHERE starboard_tracked.message_id = $1 AND starboard_tracked.starboard_channel = $2
        RETURNING starboard_post_id as "starboard_post_id: [u8; 8]""#,
        &message.as_u64().to_be_bytes(),
        &starboard_channel.as_u64().to_be_bytes()
    )
    .fetch_all(&data.db)
    .await?;

    // Handle most common states first
    if entries.len() == 1 {
        // This will not fail because we just checked that we have one entry
        let entry = entries.first().unwrap();

        let message = MessageId(u64::from_be_bytes(entry.starboard_post_id));
        starboard_channel.delete_message(ctx, message).await?;

        return Ok(());
    }

    if entries.is_empty() {
        return Ok(());
    }

    // If there are duplicate entries, delete all of them
    for entry in entries {
        let message = MessageId(u64::from_be_bytes(entry.starboard_post_id));

        starboard_channel.delete_message(ctx, message).await?;
    }

    Ok(())
}
