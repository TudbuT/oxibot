use crate::{serenity, Data, Error, EMBED_COLOR};
use dashmap::mapref::one::RefMut;
use serenity::{ChannelId, Context, Message, MessageId, PartialMember, Reaction};

// Maybe have this configurable?
const MIN_REACTIONS: u32 = 1;

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
        Some(guild) => guild.0.to_be_bytes(),
        None => return Ok(()),
    };
    let possible_channel = sqlx::query!(
        r#"SELECT starboard_channel as "starboard_channel: [u8; 8]" FROM starboard 
                    WHERE starboard.guild_id = $1 AND starboard.emoji = $2"#,
        &guild_id,
        emoji_string
    )
    .fetch_optional(&data.db)
    .await?;

    let starboard = match possible_channel {
        Some(record) => ChannelId(u64::from_be_bytes(record.starboard_channel)),
        None => return Ok(()),
    };

    match data.starboard_tracked.get_mut(&reaction.message_id) {
        Some(value) => {
            modify_existing_starboard(value, ctx).await?;
        }
        None => {
            let reactions = modify_or_insert_candidate(data, reaction.message_id);

            if reactions == MIN_REACTIONS {
                data.starboard_candidates.remove(&reaction.message_id);

                let post = create_starboard(ctx, &message, starboard, emoji_string).await?;

                data.starboard_tracked
                    .insert(reaction.message_id, (post, MIN_REACTIONS));
            }
        }
    }

    Ok(())
}

async fn modify_existing_starboard(
    mut value: RefMut<'_, MessageId, (serenity::Message, u32)>,
    ctx: &Context,
) -> Result<(), Error> {
    let (post, count) = value.value_mut();
    *count += 1;

    let content = post.content.trim_end_matches(char::is_numeric).to_string() + &count.to_string();

    post.edit(ctx, |x| x.content(content)).await?;
    Ok(())
}

fn modify_or_insert_candidate(data: &Data, message: MessageId) -> u32 {
    *data
        .starboard_candidates
        .entry(message)
        .and_modify(|x| *x += 1)
        .or_insert(1)
        .value()
}

async fn create_starboard(
    ctx: &Context,
    message: &Message,
    starboard: ChannelId,
    emoji_string: String,
) -> Result<Message, Error> {
    let link = format!("[Jump!]({})", message.link());
    let channel = message.channel(ctx).await?.to_string();

    //TODO! this should not assume that the message reactions equal MIN_REACTIONS
    let starboard_message = format!("{channel} | {emoji_string} {MIN_REACTIONS}");

    starboard
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
