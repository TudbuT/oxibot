use crate::{serenity, Data, Error};
use serenity::{ChannelId, Context, Reaction};

pub async fn handle(reaction: &Reaction, data: &Data, ctx: &Context) -> Result<(), Error> {
    let message = reaction.message_id.0;

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
    .fetch_one(&data.db)
    .await?
    .starboard_channel;

    let starboard_channel = u64::from_be_bytes(possible_channel);

    match data.starboard_tracked.get_mut(&message) {
        Some(mut value) => {
            let (post, count) = value.value_mut();
            *count += 1;

            let content =
                post.content.trim_end_matches(char::is_numeric).to_string() + &count.to_string();

            post.edit(ctx, |x| x.content(content)).await?;
        }
        None => {
            let reactions = data
                .starboard_candidates
                .get_mut(&message)
                .map(|mut x| {
                    *x += 1;
                    *x
                })
                .unwrap_or(1);

            if reactions == 3 {
                data.starboard_candidates.remove(&message);
                let content = reaction.message(ctx).await?.content;
                //TODO make this a nice embed
                let emoji = reaction.emoji.to_string();
                let msg = format!("```\n{content}```\n{emoji} Reactions: {reactions}");

                let post = ChannelId(starboard_channel)
                    .send_message(ctx, |x| x.content(msg))
                    .await?;
                data.starboard_tracked.insert(message, (post, reactions));
            }
        }
    }

    Ok(())
}
