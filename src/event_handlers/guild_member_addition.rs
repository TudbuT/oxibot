use crate::{serenity, Data, Error};
use rand::seq::SliceRandom;
use serenity::{ChannelId, Context, Member};

pub async fn handle(new_member: &Member, data: &Data, ctx: &Context) -> Result<(), Error> {
    let channel = new_member.guild_id.as_u64().to_be_bytes();

    let welcome_configs = sqlx::query!(
        r#"SELECT welcome_channel as "welcome_channel: [u8; 8]", welcome_messages 
                    FROM guild WHERE guild.discord_id = $1"#,
        &channel
    )
    .fetch_one(&data.db)
    .await?;

    let welcome_channel = match welcome_configs.welcome_channel.map(u64::from_be_bytes) {
        Some(welcome_channel) => ChannelId(welcome_channel),
        None => return Ok(()),
    };

    let message = welcome_configs
        .welcome_messages
        .choose(&mut rand::thread_rng())
        .map(|x| x.as_str())
        .unwrap_or("{} joined a server without any welcome message, how uncreative!")
        .replace("{}", new_member.display_name().as_str());

    welcome_channel
        .send_message(ctx, |x| x.content(message))
        .await?;

    Ok(())
}
