use std::mem;

use crate::{serenity, Data, Error};
use poise::serenity_prelude::{Mention, Mentionable};
use serenity::model::channel::MessageFlags;
use serenity::{ChannelId, Context, Member};
use std::fmt::Write;
use crate::database;

pub async fn handle(new_member: &Member, data: &Data, ctx: &Context) -> Result<(), Error> {
    let channel = new_member.guild_id.0 as i64;

    let welcome_configs = sqlx::query!(
        r#"SELECT welcome_channel as "welcome_channel: database::ChannelId", (welcome_messages)[1 + trunc(random() * array_length(welcome_messages, 1))::int] as welcome_message
                    FROM guild WHERE guild.discord_id = $1"#,
        &channel
    )
    .fetch_one(&data.db)
    .await?;

    let welcome_channel = match welcome_configs.welcome_channel {
        Some(welcome_channel) => welcome_channel.into_serenity(),
        None => return Ok(()),
    };

    membership_event(
        ctx,
        welcome_channel,
        welcome_configs.welcome_message,
        " joined a server without any welcome message, how uncreative!",
        new_member.mention(),
    )
    .await
}

async fn membership_event(
    ctx: &Context,
    channel: ChannelId,
    message: Option<String>,
    default_message_template: &'static str,
    user: Mention,
) -> Result<(), Error> {
    let message = message
        .map(|x| x.replace("{}", user.to_string().as_str()))
        .unwrap_or_else(|| {
            const MAX_MENTION_LEN: usize = "<@18446744073709551615>".len();
            let mut default_message =
                String::with_capacity(MAX_MENTION_LEN + default_message_template.len());
            write!(&mut default_message, "{}", user.mention()).unwrap();
            default_message.push_str(default_message_template);

            default_message
        });

    // SAFETY: we are transmuting to a u64 bitfield, and discord supports silent pings with this one
    const SILENT_FLAG: MessageFlags = unsafe { mem::transmute(4096_u64) };

    channel
        .send_message(ctx, |x| x.content(message).flags(SILENT_FLAG))
        .await?;

    Ok(())
}
