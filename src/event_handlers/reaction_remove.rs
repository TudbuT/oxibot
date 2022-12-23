use crate::{serenity, Data, Error};
use serenity::{ChannelId, Context, Reaction};

pub async fn handle(reaction: &Reaction, data: &Data, ctx: &Context) -> Result<(), Error> {
    //! We cannot check for if the user is removing a reaction because reaction.member is always None!!

    let message = reaction.message(ctx).await?;
    let emoji_string = reaction.emoji.to_string();

    let possible_post = sqlx::query!(
        r#"SELECT starboard_channel as "starboard_channel: [u8; 8]", starboard_post_id as "starboard_post_id: [u8; 8]", reaction_count FROM starboard_tracked 
                    WHERE starboard_tracked.message_id = $1 AND starboard_tracked.emoji = $2"#,
        &message.id.as_u64().to_be_bytes(),
        emoji_string
    )
    .fetch_optional(&data.db)
    .await?
    .map(|record| (ChannelId(u64::from_be_bytes(record.starboard_channel)), record.starboard_post_id , record.reaction_count));

    match possible_post {
        Some((channel_id, post_id, reactions)) => {
            let new_reactions = reactions - 1;

            let mut post = channel_id.message(ctx, u64::from_be_bytes(post_id)).await?;

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
        None => {
            data.starboard_candidates
                .entry((reaction.message_id, reaction.emoji.clone()))
                .and_modify(|x| *x -= 1);
        }
    }

    Ok(())
}
