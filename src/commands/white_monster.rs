use tracing::trace;

use crate::{Context, Error};

#[poise::command(prefix_command, slash_command)]
pub async fn wm(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let wm_counter = ctx.data().wm_counter().await;

    ctx.say(format!("White monster cans: {} / 32 :white_monster:", wm_counter)).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn wmadd(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let wm_counter = ctx.data().inc_wm_counter().await;

    trace!("Incremented white monster counter to {}", wm_counter);
    ctx.say(format!("Another white monster for the boysss! {} / 32 :white_monster:", wm_counter)).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn wmrm(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let wm_counter = ctx.data().dec_wm_counter().await;

    trace!("Decremented white monster counter to {}", wm_counter);
    ctx.say(format!("Removed one white monster from the counter! {} / 32 :white_monster:", wm_counter)).await?;
    Ok(())
}