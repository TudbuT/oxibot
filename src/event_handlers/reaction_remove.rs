use crate::{serenity, Data, Error};
use serenity::{Context, Reaction};

pub async fn handle(reaction: &Reaction, data: &Data, ctx: &Context) -> Result<(), Error> {
    let message = reaction.message_id.0;

    match data.starboard_tracked.get_mut(&message) {
        Some(mut value) => {
            let (post, count) = value.value_mut();
            *count -= 1;

            let content =
                post.content.trim_end_matches(char::is_numeric).to_string() + &count.to_string();

            post.edit(ctx, |x| x.content(content)).await?;
        }
        None => {
            data.starboard_candidates
                .entry(message)
                .and_modify(|x| *x -= 1);
        }
    }

    Ok(())
}
