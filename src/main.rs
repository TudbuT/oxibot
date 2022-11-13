mod commands;

use std::sync::Mutex;

use serenity::{
    async_trait,
    model::application::{
        command::{Command, CommandOptionType},
        interaction::{application_command::CommandDataOptionValue, Interaction},
    },
    model::prelude::*,
    prelude::*,
    Client,
};

#[macro_use]
extern crate dotenv_codegen;

const WELCOME_MESSAGES_PATH: &'static str = "./welcome-messages.json";

struct Handler {
    welcome_messages: Mutex<Vec<String>>,
}

impl Handler {
    fn new() -> Self {
        // load welcome messages on starup
        let welcome_messages = {
            let content = std::fs::read_to_string(WELCOME_MESSAGES_PATH).unwrap_or_default();

            if content.is_empty() {
                Vec::new()
            } else {
                serde_json::from_str(&content)
                    .expect("'welcome-messages.json' should be a valid JSON array")
            }
        };

        println!("welcome messages: {:?}", welcome_messages);

        Self {
            welcome_messages: Mutex::new(welcome_messages),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.online().await;
        ctx.set_activity(Activity::watching("C code become rusty"))
            .await;

        // init slash commands
        //TODO! change these to guild commands for testing
        Command::set_global_application_commands(&ctx.http, |commands| {
            commands.create_application_command(|command| {
                command.name("ping").description("simple ping-pong command")
            });

            commands.create_application_command(|command| {
                command
                    .name("addmsg")
                    .description("adds a new welcome message")
                    .create_option(|option| {
                        option
                            .name("message")
                            .description("the message to add")
                            .kind(CommandOptionType::String)
                            .required(true)
                    })
            })

            //TODO! make tag commands
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
                    "ping" => commands::misc::ping(),
                    "addmsg" => {
                        let message_arg = command
                            .data
                            .options
                            .get(0)
                            .expect("expected message option")
                            .resolved
                            .as_ref()
                            .expect("expected a String");

                        if let CommandDataOptionValue::String(message) = message_arg {
                            let mut messages = self
                                .welcome_messages
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());

                            messages.push(message.clone());

                            println!("welcome messages: {:?}", messages);

                            // save all welcome messages into a file
                            // TODO: optimize this
                            std::fs::write(
                                WELCOME_MESSAGES_PATH,
                                serde_json::to_string(&*messages).unwrap(),
                            )
                            .expect("failed to save the file");

                            format!("successfully added {:?} as a welcome message!", message)
                        } else {
                            "please provide a valid welcome message".to_string()
                        }
                    }
                    _ => "not implemented yet".to_string(),
                };

                // reply to an interaction
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
            .expect("Bot recieved a message on a guild where it isn't there");

        let channel = message
            .channel(&ctx.http)
            .await
            .expect("Http request failed")
            .guild();

        // log the message
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
        // log the message
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
    // grab the token from a .env file located in the project directory
    let token = dotenv!("TOKEN");
    let mut client = Client::builder(
        token,
        GatewayIntents::GUILDS
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT,
    )
    .event_handler(Handler::new())
    .await
    .expect("unable to start client");

    // ctrl+c handler, allows orderly shutdown of the client

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(e) = client.start().await {
        panic!("Error starting client: {e}")
    }
}
