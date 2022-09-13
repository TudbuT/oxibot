use serenity::{async_trait, model::prelude::*, prelude::*, Client};
use std::env;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.online().await;
        ctx.set_activity(Activity::watching("C code become rusty"))
            .await;
        println!(
            "Hello there! Running on {}#{}",
            ready.user.name, ready.user.discriminator
        );
    }

    async fn message(&self, ctx: Context, message: Message) {
        if message.author.bot {
            return;
        }

        log_message(&ctx, &message).await;

        if message
            .mentions_me(&ctx.http)
            .await
            .expect("cache is unavilable")
        {
            let mut args = message
                .content
                .split(' ')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty());

            args.next(); // ignore the bot mention

            let command = args.next();
            if command.map(|c| c == "ping").unwrap_or(false) {
                message
                    .channel_id
                    .say(&ctx.http, "pong!")
                    .await
                    .map(drop)
                    .unwrap_or_else(|why| eprintln!("unable to send message: {:?}", why));
            }
        }
    }
}

async fn log_message(ctx: &Context, message: &Message) {
    let author = &message.author;

    // when the bot recieves a message from a guild
    if let Some(ref guild) = message.guild_id {
        let guild = guild
            .to_partial_guild(&ctx.http)
            .await
            .expect("Bot recieved a message on a guild where he isn't there");

        let channel = message
            .channel(&ctx.http)
            .await
            .expect("Http request failed")
            .guild();

        println!(
            "User {} ({}) in guild {} ({}) channel {:?} ({:?}) says message ({}): {}",
            author.tag(),
            author.id,
            guild.name,
            guild.id,
            channel.as_ref().map(|c| &c.name),
            channel.as_ref().map(|c| &c.id),
            message.id,
            message.content
        );

    // when DMing the bot
    } else {
        println!(
            "User {} ({}) in DM says message ({}): {}",
            author.tag(),
            author.id,
            message.id,
            message.content
        );
    }
}

#[tokio::main]
async fn main() {
    let token = match env::var("TOKEN") {
        Ok(t) => t,
        Err(_) => panic!("No token was provided (env[TOKEN])"),
    };

    let mut client = Client::builder(
        token,
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES,
    )
    .event_handler(Handler)
    .await
    .expect("unable to start client");

    if let Err(e) = client.start().await {
        panic!("Error starting client: {e}")
    }
}
