use std::env;
use std::env::VarError;

use commands::{
    guild::guild, help::help, ping::pong, starboard::starboard, tags::*, welcome::welcome,
};
use dashmap::DashMap;
use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::event_handlers::event_handler;
use poise::serenity_prelude as serenity;
use poise::Prefix;
use serenity::{Activity, GatewayIntents, Message};

mod commands;
mod event_handlers;

const INTENTS: GatewayIntents = GatewayIntents::non_privileged()
    .union(GatewayIntents::MESSAGE_CONTENT)
    .union(GatewayIntents::GUILD_MEMBERS);

type Context<'a> = poise::Context<'a, Data, Error>;
type Error = Box<dyn std::error::Error + Send + Sync>;

// Data shared across commands and events
pub struct Data {
    pub db: PgPool,
    pub starboard_candidates: DashMap<u64, u32>,
    pub starboard_tracked: DashMap<u64, (Message, u32)>,
}

#[tokio::main]
async fn main() {
    let commands = vec![
        register_commands(),
        help(),
        pong(),
        starboard(),
        guild(),
        welcome(),
        tag_edit(),
        tag_list(),
        tags(),
    ];

    if matches!(dotenv(), Err(dotenvy::Error::Io(_))) && !not_using_dotenv() {
        println!("You have not included a .env file! If this is intentional you can disable this warning with `DISABLE_NO_DOTENV_WARNING=1`")
    }

    tracing_subscriber::fmt::init();

    // If we used dotenv! you would have to recompile to update these
    let token = env::var("DISCORD_TOKEN").expect("No discord token found in environment variables");
    let database_url =
        env::var("DATABASE_URL").expect("No database url found in environment variables");
    let (primary_prefix, addition_prefixes) = parse_prefixes();

    let db = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to database!");

    sqlx::migrate!()
        .run(&db)
        .await
        .expect("Unable to apply migrations!");

    let data = Data {
        db,
        starboard_candidates: Default::default(),
        starboard_tracked: Default::default(),
    };

    // init settings for the framework
    let framework_builder = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: primary_prefix,
                additional_prefixes: addition_prefixes,
                edit_tracker: Some(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(120),
                )),
                ..Default::default()
            },
            commands,
            event_handler: |ctx, event, _framework, data| Box::pin(event_handler(ctx, event, data)),
            ..Default::default()
        })
        .token(token)
        .intents(INTENTS)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                ctx.set_activity(Activity::watching("C code become rusty"))
                    .await;

                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(data)
            })
        });

    // actually init the framework
    let framework = framework_builder
        .build()
        .await
        .expect("Cannot create the bot framework!");

    // ctrl+c handler
    let shard_handler = framework.shard_manager().clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Couldn't register a ctrl+c handler!");
        tracing::info!("Shutting down oxibot!");
        shard_handler.lock().await.shutdown_all().await;
        //TODO! Close the pool without cloning anything
    });

    tracing::info!("Starting oxibot!");
    framework.start().await.unwrap();
}

fn not_using_dotenv() -> bool {
    match env::var("DISABLE_NO_DOTENV_WARNING") {
        Ok(value) if value == "1" => true,
        Ok(value) if value == "0" => false,
        Ok(_) => {
            panic!("DISABLE_NO_DOTENV_WARNING environment variable is equal to something other then 1 or 0")
        }
        Err(_) => false,
    }
}

fn parse_prefixes() -> (Option<String>, Vec<Prefix>) {
    let unparsed = match env::var("PREFIXES") {
        Ok(unparsed) => unparsed,
        Err(VarError::NotPresent) => return (None, Vec::new()),
        _ => panic!("Could not handle the environment variable for prefixes"),
    };

    let mut split = unparsed.split(' ').map(|x| x.to_string());

    let first = split
        .next()
        .expect("Could not parse prefixes from environment variables");

    // We need to leak these strings since `Prefix::Literal` only accepts `&'static str` for some reason
    let split = split
        .map(|x| Box::leak(Box::new(x)))
        .map(|x| Prefix::Literal(x));

    (Some(first), split.collect())
}

#[poise::command(prefix_command, hide_in_help, owners_only)]
async fn register_commands(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}
