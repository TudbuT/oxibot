macro_rules! database_id {
    ($i:ident) => {
        #[derive(sqlx::Type, Copy, Clone, Debug, Eq, Hash, PartialEq)]
        #[sqlx(transparent)]
        pub struct $i(i64);

        impl From<$i> for poise::serenity_prelude::id::$i {
            fn from($i(id): $i) -> Self {
                Self(id as u64)
            }
        }
    };
}

database_id!(ChannelId);
database_id!(MessageId);
database_id!(GuildId);
database_id!(UserId);
