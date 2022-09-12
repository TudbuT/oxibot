use serenity::{Client, prelude::GatewayIntents};
use std::env;

#[tokio::main]
async fn main() {
    let token = match env::var("TOKEN") {
        Ok(t) => t,
        Err(_) => panic!("No token was provided (env[TOKEN])"),
    };
    let mut client = Client::builder(token, GatewayIntents::GUILDS).await.expect("unable to start client");
    if let Err(e) = client.start().await {
        panic!("Error starting client: {e}")
    }
}
