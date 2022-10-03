use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;

#[group]
#[commands(mc)]
struct Minecraft;

#[command]
pub async fn mc(ctx: &Context, msg: &Message) -> CommandResult {
    use std::process::Command;

    let res = Command::new("./send.sh")
        .arg(&msg.content)
        .spawn();
    
    msg.reply(ctx, 
        if let Ok(_) = res { format!("Success '{}'", &msg.content) }
        else { "Service unavailable".to_string() }
    ).await?;
    Ok(())
}