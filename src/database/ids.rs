macro_rules! database_id {
    ($i:ident) => {
        #[derive(sqlx::Type, Copy, Clone, Debug, Eq, Hash, PartialEq)]
        #[sqlx(transparent)]
        pub struct $i(pub i64);

        impl $i {
            pub fn into_serenity(self) -> poise::serenity_prelude::id::$i {
                poise::serenity_prelude::id::$i(self.0 as u64)
            }
        }

        impl From<$i> for poise::serenity_prelude::id::$i {
            fn from(id: $i) -> Self {
                id.into_serenity()
            }
        }

        impl From<poise::serenity_prelude::id::$i> for $i {
            fn from(poise::serenity_prelude::id::$i(id): poise::serenity_prelude::id::$i) -> Self {
                Self(id as i64)
            }
        }
    };
}

database_id!(ChannelId);
database_id!(MessageId);
database_id!(GuildId);
database_id!(UserId);
