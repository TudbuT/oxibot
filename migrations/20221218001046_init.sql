CREATE TABLE guild(
  discord_id BYTEA PRIMARY KEY,
  welcome_channel BYTEA,
  welcome_messages TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[]
);

CREATE TABLE starboard(
  guild_id BYTEA,
  emoji TEXT,
  starboard_channel BYTEA NOT NULL,

  PRIMARY KEY(guild_id, emoji),
  CONSTRAINT fk_guild FOREIGN KEY(guild_id) REFERENCES guild(discord_id)
);

CREATE TABLE tag(
  guild_id BYTEA,
  command_name TEXT,
  tag_description TEXT NOT NULL,

  PRIMARY KEY(guild_id, command_name),
  CONSTRAINT fk_guild FOREIGN KEY(guild_id) REFERENCES guild(discord_id)
);