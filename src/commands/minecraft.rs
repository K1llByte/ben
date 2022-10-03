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
        .arg("list")
        .spawn();
    
    msg.reply(ctx, 
        if let Ok(_) = res { "Success" }
        else { "Service unavailable" }
    ).await?;
    Ok(())
}