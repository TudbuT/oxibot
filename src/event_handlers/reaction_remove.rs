use crate::{serenity, Data, Error};
use serenity::{Context, Reaction};

pub async fn handle(reaction: &Reaction, data: &Data, ctx: &Context) -> Result<(), Error> {
    match data.starboard_tracked.get_mut(&reaction.message_id) {
        Some(mut value) => {
            let (post, count) = value.value_mut();
            *count -= 1;

            let content =
                post.content.trim_end_matches(char::is_numeric).to_string() + &count.to_string();

            post.edit(ctx, |x| x.content(content)).await?;
        }
        None => {
            data.starboard_candidates
                .entry(reaction.message_id)
                .and_modify(|x| *x -= 1);
        }
    }

    Ok(())
}
