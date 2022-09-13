use serenity::{
    async_trait,
    model::application::interaction::Interaction,
    model::prelude::{command::Command, *},
    prelude::*,
    Client,
};

#[macro_use]
extern crate dotenv_codegen;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.online().await;
        ctx.set_activity(Activity::watching("C code become rusty"))
            .await;

        Command::set_global_application_commands(&ctx.http, |commands| {
            commands.create_application_command(|command| {
                command.name("ping").description("simple ping-pong command")
            })
        })
        .await
        .expect("Recieved an Http error");

        println!("Hello there! Running on {}", ready.user.tag());
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        log_interaction(&ctx, &interaction).await;

        match interaction {
            Interaction::ApplicationCommand(command) => {
                let reply = match command.data.name.as_str() {
                    "ping" => "pong!",
                    _ => "not implemented yet",
                };

                command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(interaction::InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| message.content(reply))
                    })
                    .await
                    .unwrap_or_else(|why| println!("Unable to respond to slash command: {:?}", why))
            }

            _ => return,
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
        if message.author.bot {
            return;
        }

        log_message(&ctx, message).await;
    }
}

async fn log_interaction(_ctx: &Context, interaction: &Interaction) {
    println!("{:#?}", interaction)
}

async fn log_message(ctx: &Context, message: Message) {
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
    let token = dotenv!("TOKEN");

    let mut client = Client::builder(
        token,
        GatewayIntents::GUILDS
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT,
    )
    .event_handler(Handler)
    .await
    .expect("unable to start client");

    if let Err(e) = client.start().await {
        panic!("Error starting client: {e}")
    }
}
