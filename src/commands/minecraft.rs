use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, Args};

#[group]
#[commands(mc)]
struct Minecraft;

#[command]
pub async fn mc(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    use std::process::Command;
    if let Some(mc_cmd) = args.remains() {
        let res = Command::new("./send.sh")
            .arg(mc_cmd)
            .spawn();
        
        msg.reply(ctx, 
            if let Ok(_) = res { format!("Success '{:?}'", mc_cmd) }
            else { "Service unavailable".to_string() }
        ).await?;
    }
    Ok(())
}