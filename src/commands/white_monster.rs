use poise::serenity_prelude::UserId;
use tracing::trace;

use crate::{Context, Error};

#[poise::command(prefix_command, slash_command)]
pub async fn wm(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let mut wm_data = ctx.data().wm_counters().await;
    wm_data.1.sort_by(|a, b| b.1.cmp(&a.1));

    let mut output = format!("White monster cans: {} / 32\n", wm_data.0);
    for (user_id, count) in wm_data.1 {
        if let Some(cached_user) = ctx.cache().user(user_id) {
            output.push_str(format!("- **{}** has {}\n",  cached_user.display_name(), count).as_str());
        }
        else {
            let user = ctx.http().get_user(UserId::new(user_id)).await.unwrap();
            output.push_str(format!("- **{}** has {}\n",  user.display_name(), count).as_str());
        };
    }
    ctx.say(output).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn wmadd(
    ctx: Context<'_>,
    #[description = "Number of white monsters to add (default: 1)"]
    value: Option<u32>
) -> Result<(), Error> {
    let wm_counter = ctx.data().inc_wm_counter(
        ctx.author().id.get(),
        value.unwrap_or(1)
    ).await;

    trace!("Incremented white monster counter to {}", wm_counter);
    ctx.say(format!("Another white monster for the boysss! {} / 32", wm_counter)).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn wmrm(
    ctx: Context<'_>,
    #[description = "Number of white monsters to remove (default: 1)"]
    value: Option<u32>
) -> Result<(), Error> {
    let wm_counter = ctx.data().dec_wm_counter(
        ctx.author().id.get(),
        value.unwrap_or(1)
    ).await;

    trace!("Decremented white monster counter to {}", wm_counter);
    ctx.say(format!("Removed one white monster from the counter! {} / 32", wm_counter)).await?;
    Ok(())
}