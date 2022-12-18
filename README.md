# Oxibot
Oxibot is a general purpose discord bot written in Rust aimed.

# Running locally

To run locally, you can create a `.env` file with the following options:

```
DISCORD_TOKEN="token" # From discord https://discord.com/developers/applications
DATABASE_URL="postgres://user:password@localhost:5432/database_name" # You might have to url encode your password
PREFIXES="" # A space seperated list of the prefixes you want the bot to have
RUST_LOG=INFO
RUST_BACKTRACE=1
``` 

Make sure your discord bot also has the privileged gateway intents for message content and server members.

To set up your database you can use sqlx migrations:

```
cargo install sqlx-cli
sqlx database create
sqlx migrate run  # This step is optional since it is done already at the startup of the application
```

And you are done!
