use poise::serenity_prelude::{self as serenity};
use std::{
    collections::HashSet, sync::Arc, time::Duration
};
use tracing_subscriber;
use tracing::{info, error};

use crate::config::Config;

mod commands;
mod model;
mod config;

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, model::Model, Error>;

async fn on_error(error: poise::FrameworkError<'_, model::Model, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Load the .env file
    dotenv::dotenv().ok();

    // Initialize logger
     tracing_subscriber::fmt()
        // .with_env_filter("monster_bot=debug,tower_http=info")
        .init();

    // Login with a bot token from the environment
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let config = Config {
        cmc_api_key: std::env::var("CMC_API_KEY").expect("Expected an api key in the environment"),
        ..Config::default()
    };

    #[allow(unused_mut)]
    let mut owners = HashSet::new();
    // owners.insert(UserId::new(181002804813496320));

    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    let options = poise::FrameworkOptions {
        commands: vec![
            commands::help(),
            commands::debug(),
            // White Monster
            commands::wm(),
            commands::wmadd(),
            commands::wmrm(),
            // Finance
            commands::bank(),
            commands::give(),
            commands::bless(),
            commands::leaderboard(),
            commands::price(),
            commands::portfolio(),
            commands::buy(),
            commands::sell(),
            // commands::sellall(),
            commands::coin(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("!".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            ..Default::default()
        },
        // The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),
        // This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                info!("{} executed command {}", ctx.author().display_name(), ctx.command().qualified_name);
            })
        },
        owners,
        ..Default::default()
    };

    // Initialize discord poise framework
    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                info!("Connected as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(model::Model::new(config).await)
            })
        })
        .options(options)
        .build();

    // Set gateway intents, which decides what events the bot will be notified about
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap()
}
