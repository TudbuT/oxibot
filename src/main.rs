use serenity::{Client, prelude::*, model::prelude::*, async_trait};
use std::env;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.online().await;
        ctx.set_activity(Activity::watching("C code become rusty")).await;
        println!("Hello there! Running on {}#{}", ready.user.name, ready.user.discriminator);
    }
    
    async fn message(&self, ctx: Context, message: Message) {
        if {
            if let Some(guild) = message.guild_id {
                if let (Ok(guild), Ok(Some(channel))) = (guild.to_partial_guild(&ctx.http).await, message.channel_id.to_channel(&ctx.http).await.map(|x| x.guild())) {
                    println!(
                        "User {}#{} ({}) in guild {} ({}) channel {} ({}) says message ({}): {}",
                        message.author.name, message.author.discriminator, message.author.id, 
                        guild.name, guild.id,
                        channel.name, channel.id,
                        message.id, message.content
                    );
                    false
                }
                else {true}
            } else {true}
        } {
            println!(
                "User {}#{} ({}) in DM says message ({}): {}",
                message.author.name, message.author.discriminator, message.author.id, 
                message.id, message.content
            );
        }
        if let Ok(true) = message.mentions_me(&ctx.http).await {
            let mut args = message.content.split(" ");
            let _ = args.next();
            if let Some(arg0) = args.next() {
                if arg0 == "ping" {
                    let _ = message.channel_id.say(ctx.http, "Pong!").await;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let token = match env::var("TOKEN") {
        Ok(t) => t,
        Err(_) => panic!("No token was provided (env[TOKEN])"),
    };
    let mut client = Client::builder(token, GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES)
        .event_handler(Handler)
        .await.expect("unable to start client");
    if let Err(e) = client.start().await {
        panic!("Error starting client: {e}")
    }
}
