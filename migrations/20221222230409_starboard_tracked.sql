CREATE TABLE starboard_tracked(
  message_id BIGINT,
  emoji TEXT,
  starboard_channel BIGINT NOT NULL,
  starboard_post_id BIGINT NOT NULL UNIQUE,
  reaction_count INTEGER NOT NULL,

  PRIMARY KEY(message_id, emoji)
);