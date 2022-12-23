use crate::{serenity, Data, Error};
use serenity::{ChannelId, Context, MessageId};

pub async fn handle(deleted_message: &MessageId, data: &Data, ctx: &Context) -> Result<(), Error> {
    data.starboard_candidates
        .retain(|(id, _), _| id == deleted_message);

    let tracked = sqlx::query!(
        r#"DELETE FROM starboard_tracked WHERE starboard_tracked.message_id = $1 
        RETURNING starboard_post_id as "starboard_post_id: [u8; 8]", starboard_channel as "starboard_channel: [u8; 8]""#,
        &deleted_message.as_u64().to_be_bytes(),
    )
    .fetch_all(&data.db)
    .await?;

    for ele in tracked {
        let channel = ChannelId(u64::from_be_bytes(ele.starboard_channel));
        let message = MessageId(u64::from_be_bytes(ele.starboard_post_id));

        channel.delete_message(ctx, message).await?;
    }

    Ok(())
}
