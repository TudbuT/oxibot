use serenity::{Client, prelude::*, framework::{StandardFramework, standard::{macros::command, CommandResult}}, model::prelude::*};
use std::env;

struct Handler;

impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    let token = match env::var("TOKEN") {
        Ok(t) => t,
        Err(_) => panic!("No token was provided (env[TOKEN])"),
    };
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("oxi!"));
    let mut client = Client::builder(token, GatewayIntents::GUILDS)
        .event_handler(Handler)
        .framework(framework)
        .await.expect("unable to start client");
    if let Err(e) = client.start().await {
        panic!("Error starting client: {e}")
    }
}

#[command]
async fn test(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}
