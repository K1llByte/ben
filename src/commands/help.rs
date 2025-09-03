use crate::{Context, Error};

#[poise::command(prefix_command, slash_command)]
pub async fn help(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let help_message = "Available commands: \n\
        ```\n\
        wmadd - Increments the counter of white monster cans\n\
        wmrm  - Decrements the counter of white monster cans\n\
        wm    - Displays the current counter of white monster cans\n\
        ```";

    ctx.say(help_message).await?;
    Ok(())
}