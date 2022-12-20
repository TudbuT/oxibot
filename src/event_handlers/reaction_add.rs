use crate::{serenity, Data, Error};
use dashmap::mapref::one::RefMut;
use poise::serenity_prelude::{Color, PartialMember};
use serenity::{ChannelId, Context, Reaction};

// Maybe have this configurable?
const MIN_REACTIONS: u32 = 3;

pub async fn handle(reaction: &Reaction, data: &Data, ctx: &Context) -> Result<(), Error> {
    let message = reaction.message_id.0;

    let reactor = match reaction.member.as_ref() {
        Some(PartialMember {
            user: Some(user), ..
        }) => user,
        _ => return Ok(()),
    };

    if &reaction.message(ctx).await?.author == reactor {
        return Ok(());
    }

    let guild = match reaction.guild_id {
        Some(guild) => guild.0,
        None => return Ok(()),
    };
    let possible_channel = sqlx::query!(
        r#"SELECT starboard_channel as "starboard_channel: [u8; 8]" FROM starboard 
                    WHERE starboard.guild_id = $1 AND starboard.emoji = $2"#,
        &guild.to_be_bytes(),
        reaction.emoji.to_string()
    )
    .fetch_optional(&data.db)
    .await?;

    let starboard_channel = match possible_channel {
        Some(record) => u64::from_be_bytes(record.starboard_channel),
        None => return Ok(()),
    };

    match data.starboard_tracked.get_mut(&message) {
        Some(value) => {
            modify_existing_starboard(value, ctx).await?;
        }
        None => {
            let reactions = modify_or_insert_candidate(data, message);

            if reactions == MIN_REACTIONS {
                create_starboard(data, message, reaction, ctx, starboard_channel).await?;
            }
        }
    }

    Ok(())
}

async fn modify_existing_starboard(
    mut value: RefMut<'_, u64, (serenity::Message, u32)>,
    ctx: &Context,
) -> Result<(), Error> {
    let (post, count) = value.value_mut();
    *count += 1;

    let content = post.content.trim_end_matches(char::is_numeric).to_string() + &count.to_string();

    post.edit(ctx, |x| x.content(content)).await?;
    Ok(())
}

fn modify_or_insert_candidate(data: &Data, message: u64) -> u32 {
    *data
        .starboard_candidates
        .entry(message)
        .and_modify(|x| *x += 1)
        .or_insert(1)
        .value()
}

async fn create_starboard(
    data: &Data,
    message_id: u64,
    reaction: &Reaction,
    ctx: &Context,
    starboard_channel: u64,
) -> Result<(), Error> {
    data.starboard_candidates.remove(&message_id);

    let message = reaction.message(ctx).await?;
    let link = format!("[Jump!]({})", message.link());
    let channel = message.channel(ctx).await?.to_string();
    let emoji = reaction.emoji.to_string();
    //TODO! this should not assume that the message reactions equal MIN_REACTIONS
    let starboard_message = format!("{channel} | {emoji} {MIN_REACTIONS}");

    let post = ChannelId(starboard_channel)
        .send_message(ctx, |m| {
            m.content(starboard_message).embed(|e| {
                e.author(|a| {
                    a.icon_url(message.author.face())
                        .name(message.author.name.clone())
                })
                .description(message.content_safe(ctx))
                .field("Source", link, false)
                .color(Color::from_rgb(255, 172, 51))
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

    data.starboard_tracked
        .insert(message_id, (post, MIN_REACTIONS));

    Ok(())
}
