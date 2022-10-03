use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, Args};

#[group]
#[commands(mc,mcstart)]
struct Minecraft;

#[command]
pub async fn mc(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    use std::process::Command;
    if let Some(start) = args.current() {
        if start == "start" {
            // Start command
            let res = Command::new("./run.sh")
                .spawn();
        
            msg.reply(ctx, 
                if let Ok(_) = res { format!("Success") }
                else { "Service unavailable".to_string() }
            ).await?;
        }
    }
    else if let Some(mc_cmd) = args.remains() {
        // Other commands
        let res = Command::new("./send.sh")
            .arg(mc_cmd)
            .spawn();
        
        msg.reply(ctx, 
            if let Ok(_) = res { format!("Success") }
            else { "Service unavailable".to_string() }
        ).await?;
    }
    Ok(())
}

#[command]
pub async fn mcstart(ctx: &Context, msg: &Message) -> CommandResult {
    use std::process::Command;
    if let Some(mc_cmd) = args.remains() {
        let res = Command::new("./start.sh")
            .arg(mc_cmd)
            .spawn();
        
        msg.reply(ctx, 
            if let Ok(_) = res { format!("Success") }
            else { "Service unavailable".to_string() }
        ).await?;
    }
    Ok(())
}