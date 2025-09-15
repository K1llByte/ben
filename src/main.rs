use poise::serenity_prelude::{self as serenity};
use std::{collections::HashSet, sync::Arc, time::Duration};
use tracing::{error, info};
use tracing_subscriber::{self, EnvFilter};

use crate::{config::Config, model::ModelError};

mod commands;
mod config;
mod model;
mod permissions;

// Types used by all command functions
type Error = anyhow::Error;
type Context<'a> = poise::Context<'a, model::Model, anyhow::Error>;

async fn on_error(error: poise::FrameworkError<'_, model::Model, anyhow::Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error);
            if let Some(ModelError::UnexpectedError) = error.downcast_ref::<ModelError>() {
                ctx.say("Unexpected error").await.unwrap();
            }
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
        // .with_env_filter(EnvFilter::from_default_env())
        .with_env_filter("ben=trace")
        .init();

    // Load bot config from toml file
    let config = Config::from_file(".config.toml").await.unwrap();
    let discord_token = config.discord_token.clone();

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
            commands::sellall(),
            commands::coin(),
            commands::daily(),
        ],
        // If true, discord bot owner account will be added automatically
        initialize_owners: false,
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
                info!(
                    "{} executed command {}",
                    ctx.author().display_name(),
                    ctx.command().qualified_name
                );
            })
        },
        ..Default::default()
    };

    // Initialize discord poise framework
    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                info!("Connected as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(model::Model::new(config).await?)
            })
        })
        .options(options)
        .build();

    // Set gateway intents, which decides what events the bot will be notified about
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(discord_token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap()
}
