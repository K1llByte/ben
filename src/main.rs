mod commands;
use commands::minecraft::*;
mod handlers;
use handlers::*;

use std::env;
use dotenv::dotenv;
use serenity::prelude::*;
use serenity::framework::standard::StandardFramework;


#[tokio::main]
async fn main() {
    // Load dotenv and set token
    dotenv().ok();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Missing discord app token!");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Setup framework
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("+")) // set the bot's prefix to "~"
        .group(&MINECRAFT_GROUP);

    // Create a new instance of the Client
    let mut client =
        Client::builder(&token, intents)
            .event_handler(Handler)
            .framework(framework)
            .await
            .expect("Err creating client");

    // Start a single shard, and start listening to events.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}