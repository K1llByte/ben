use tracing::trace;

use crate::{Context, permissions::*};

/// Help command
#[poise::command(prefix_command, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> anyhow::Result<()> {
    let config = poise::builtins::HelpConfiguration {
        ephemeral: false,
        extra_text_at_bottom: "\
Type !help <command> for more info on a command.",
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

/// ADMIN COMMAND: Debug command used for testing
#[poise::command(prefix_command, slash_command, owners_only, check = "is_admin")]
pub async fn debug(ctx: Context<'_>) -> anyhow::Result<()> {
    trace!("This is a trace log");
    ctx.say(format!("{:?}", ctx.framework().options().owners))
        .await?;
    Ok(())
}
