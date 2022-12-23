CREATE TABLE starboard_tracked(
  message_id BYTEA,
  emoji TEXT,
  starboard_channel BYTEA NOT NULL,
  starboard_post_id BYTEA NOT NULL UNIQUE,
  reaction_count INTEGER NOT NULL,

  PRIMARY KEY(message_id, emoji)
);