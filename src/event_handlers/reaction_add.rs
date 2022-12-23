use crate::{serenity, Data, Error, EMBED_COLOR};
use serenity::{ChannelId, Context, Message, MessageId, PartialMember, Reaction, ReactionType};

pub async fn handle(reaction: &Reaction, data: &Data, ctx: &Context) -> Result<(), Error> {
    let reactor = match reaction.member.as_ref() {
        Some(PartialMember {
            user: Some(user), ..
        }) => user,
        _ => return Ok(()),
    };

    let message = reaction.message(ctx).await?;
    let emoji_string = reaction.emoji.to_string();

    if &message.author == reactor {
        return Ok(());
    }

    let guild_id = match reaction.guild_id {
        Some(guild) => guild.as_u64().to_be_bytes(),
        None => return Ok(()),
    };

    let possible_starboard = sqlx::query!(
        r#"SELECT starboard_channel as "starboard_channel: [u8; 8]", min_reactions FROM starboard 
                    WHERE starboard.guild_id = $1 AND starboard.emoji = $2"#,
        &guild_id,
        emoji_string
    )
    .fetch_optional(&data.db)
    .await?
    .map(|record| (record.starboard_channel, record.min_reactions));

    let possible_post = sqlx::query!(
        r#"SELECT starboard_channel as "starboard_channel: [u8; 8]", starboard_post_id as "starboard_post_id: [u8; 8]", reaction_count FROM starboard_tracked 
                    WHERE starboard_tracked.message_id = $1 AND starboard_tracked.emoji = $2"#,
        &message.id.as_u64().to_be_bytes(),
        emoji_string
    )
    .fetch_optional(&data.db)
    .await?
    .map(|record| (ChannelId(u64::from_be_bytes(record.starboard_channel)), record.starboard_post_id , record.reaction_count));

    match (possible_starboard, possible_post) {
        (_, Some((channel, post_id, reactions))) => {
            let mut post = channel.message(ctx, u64::from_be_bytes(post_id)).await?;

            let new_reactions = reactions + 1;

            sqlx::query!(
                "UPDATE starboard_tracked SET reaction_count = $3 WHERE starboard_tracked.starboard_post_id = $1 AND starboard_tracked.emoji = $2",
                &post_id,
                emoji_string,
                new_reactions
            ).execute(&data.db)
            .await?;

            let content = post.content.trim_end_matches(char::is_numeric).to_string()
                + &new_reactions.to_string();

            post.edit(ctx, |x| x.content(content)).await?;
        }
        (Some((starboard_channel, min_reactions)), None) => {
            let candidate_key = (reaction.message_id, reaction.emoji.clone());

            let reactions = modify_or_insert_candidate(data, candidate_key.clone());

            if reactions == min_reactions as u32 {
                data.starboard_candidates.remove(&candidate_key);

                let post = create_starboard(
                    ctx,
                    &message,
                    ChannelId(u64::from_be_bytes(starboard_channel)),
                    &emoji_string,
                    min_reactions,
                )
                .await?;

                sqlx::query!(
                    r#"INSERT INTO starboard_tracked 
                    (message_id, emoji, starboard_channel, starboard_post_id, reaction_count) VALUES ($1, $2, $3, $4, $5)"#,
                    &message.id.as_u64().to_be_bytes(),
                    emoji_string,
                    &starboard_channel,
                    &post.id.as_u64().to_be_bytes(),
                    min_reactions
                ).execute(&data.db)
                .await?;
            }
        }
        (None, None) => (),
    }

    Ok(())
}

fn modify_or_insert_candidate(data: &Data, candidate_key: (MessageId, ReactionType)) -> u32 {
    *data
        .starboard_candidates
        .entry(candidate_key)
        .and_modify(|x| *x += 1)
        .or_insert(1)
        .value()
}

async fn create_starboard(
    ctx: &Context,
    message: &Message,
    starboard_channel: ChannelId,
    emoji_string: &str,
    current_reactions: i32,
) -> Result<Message, Error> {
    let link = format!("[Jump!]({})", message.link());
    let channel = message.channel(ctx).await?.to_string();

    let starboard_message = format!("{channel} | {emoji_string} {current_reactions}");

    starboard_channel
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
        .await
        .map_err(|x| x.into())
}
