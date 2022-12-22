ALTER TABLE starboard ADD min_reactions INTEGER NOT NULL DEFAULT 3;

ALTER TABLE starboard ADD CONSTRAINT not_zero_or_neg_min_reactions CHECK (min_reactions > 0);