use std::collections::HashMap;

use poise::serenity_prelude::{Client, Mention, User, UserId};
use tracing_subscriber::fmt::format;

use crate::{commands::get_user_name, Context, Error};

/// Displays current money balance (in euros).
#[poise::command(prefix_command, slash_command, category = "Finance", aliases("balance"))]
pub async fn bank(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let user_id = ctx.author().id;

    if let Some(balance) = ctx.data().balance(user_id.get()).await {
        ctx.say(
            format!("**{}** has `{}` euros", 
                get_user_name(&ctx, user_id).await,
                balance
            )
        ).await.unwrap();
    }
    else {
        let created = ctx.data().create_bank_account(user_id.get()).await;
        if created {
            ctx.say(
                format!("__Created bank account!__\n**{}** - `{}` (eur)", 
                    get_user_name(&ctx, user_id).await,
                    0
                )
            ).await.unwrap();
        }
        else {
            ctx.say(
                "Unexpected error creating bank account!"
            ).await.unwrap();
        }
    }

    Ok(())
}

/// Give money (in euros) to another user.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn give(
    ctx: Context<'_>,
    dst_user: User,
    amount: f64,
) -> Result<(), Error> {
    // Check if source user has a bank account
    let src_user_id = ctx.author().id.get();
    if !ctx.data().bank_account_exists(src_user_id).await {
        ctx.say(format!("User **{}** has no bank account", ctx.author().name))
            .await
            .unwrap();
        return Ok(());
    }
    
    // Check if destination user has a bank account
    let dst_user_id = dst_user.id.get();
    if !ctx.data().bank_account_exists(dst_user_id).await {
        ctx.say(format!("User **{}** has no bank account", dst_user.name))
            .await
            .unwrap();
        return Ok(());
    }

    // Check if is a valid positive amount
    if amount <= 0f64 {
        ctx.say("Must be a positive amount!")
            .await
            .unwrap();
        return Ok(());
    }

    if let Some((_, _)) = ctx.data().give(src_user_id, dst_user_id, amount).await {
        ctx.say(format!("{} gave {} `{}` euros.", ctx.author().name, dst_user.name, amount))
            .await
            .unwrap();
    }
    else {
        ctx.say("Insuficient funds!")
            .await
            .unwrap();
    }

    Ok(())
}

/// ADMIN COMMAND: Inject money (in euros) to another user.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn bless(
    ctx: Context<'_>,
    dst_user: User,
    amount: f64,

) -> Result<(), Error> {
    if let Some(_) = ctx.data().bless(dst_user.id.get(), amount).await {
        ctx.say(
            format!(
                "**{}**, you were blessed with `{}` euros, amen :pray:",
                dst_user.name,
                amount
            )
        )
        .await
        .unwrap();
    }
    else {
        ctx.say(format!("User **{}** has no bank account", dst_user.name))
            .await
            .unwrap();
    }

    Ok(())
}

/// Bank leaderboard. Who's the wealthiest.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn leaderboard(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let bank_data = ctx.data().leaderboard().await;

    let mut output = String::new();
    for (user_id, balance) in bank_data {
        output.push_str(
            format!(
                "- **{}** has `{}` euros\n", 
                get_user_name(&ctx, UserId::new(user_id)).await,
                balance
            ).as_str()
        );
    }
    ctx.say(output).await?;

    Ok(())
}

/// Displays the current price for a specific coin. For more info go to https://coinmarketcap.com/
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn price(
    ctx: Context<'_>,
    #[description = "Crypto currency symbol (ie: btc, eth, ...)"]
    coin_symbol: String,
) -> Result<(), Error> {
    if let Some(coin_info) = ctx.data().coin_info(&coin_symbol).await {
        ctx.say(format!("**Name:** `{}`\n**Current Price:** `{}` euros", coin_info.name, coin_info.current_price)).await?;
    }
    else {
        ctx.say(format!("Could not get coin info")).await?;
    }

    Ok(())
}

/// Displays list of owned coins amount, the profit percentage, and absolute profit in euros.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn portfolio(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let portfolio_opt = ctx.data().portfolio(ctx.author().id.get()).await;
    
    if let Some(portfolio) = portfolio_opt {
        let mut portfolio_str = "Portfolio:\n".to_string();
        for (coin_symbol, total_amount, total_value) in &portfolio {
            portfolio_str.push_str(
                format!(
                    "- Symbol: **{}**, Total Amount: `{}` Total Value: `{}` euros\n",
                    coin_symbol,
                    total_amount,
                    total_value
                ).as_str()
            );
        }

        ctx.say(portfolio_str).await.unwrap();
    }
    else {
        ctx.say("Could not get portfolio data").await.unwrap();
        
    }

    Ok(())
}

/// Buy crypto currency in euros, if successful prints amount of coins bought.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn buy(
    ctx: Context<'_>,
    #[description = "Crypto currency symbol (ie: btc, eth, ...)"]
    coin_symbol: String,
    #[description = "Value in euros of the amount of crypto you want to buy"]
    value: f64,
) -> Result<(), Error> {
    
    let amount_and_price_opt = ctx.data().buy(ctx.author().id.get(), &coin_symbol, value).await;

    if let Some(amount_and_price) = amount_and_price_opt {
        ctx.say(format!("Successfully bought `{}` {} at {} euros", amount_and_price.0, coin_symbol.to_uppercase(), amount_and_price.1)).await.unwrap();
    }
    else {
        ctx.say("Could not complete transaction").await.unwrap();
    }

    Ok(())
}

/// Sell crypto currency in euros, if successful prints amount of coins bought.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn sell(
    ctx: Context<'_>,
    #[description = "Crypto currency symbol (ie: btc, eth, ...)"]
    coin_symbol: String,
    #[description = "Value in euros of the amount of crypto you want to sell"]
    value: f64,
) -> Result<(), Error> {

    let amount_and_price_opt = ctx.data().sell(ctx.author().id.get(), &coin_symbol, value).await;
    
    if let Some(amount_and_price) = amount_and_price_opt {
        ctx.say(format!("Successfully sold `{}` {} at {} euros", amount_and_price.0, coin_symbol.to_uppercase(), amount_and_price.1)).await.unwrap();
    }
    else {
        ctx.say("Could not complete transaction").await.unwrap();
    }

    Ok(())
}

/// Sell crypto currency in euros, if successful prints amount of coins bought.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn sellall(
    ctx: Context<'_>,
    #[description = "Crypto currency symbol (ie: btc, eth, ...)"]
    coin_symbol: String,
) -> Result<(), Error> {

    let amount_and_price_opt = ctx.data().sell_all(ctx.author().id.get(), &coin_symbol).await;
    
    if let Some(amount_and_price) = amount_and_price_opt {
        ctx.say(format!("Successfully sold `{}` {} at {} euros", amount_and_price.0, coin_symbol.to_uppercase(), amount_and_price.1)).await.unwrap();
    }
    else {
        ctx.say("Could not complete transaction").await.unwrap();
    }

    Ok(())
}

/// Bet on heads or tails.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn coin(
    ctx: Context<'_>,
    #[description = "Heads or tails"]
    choice: String,
    #[description = "Bet amount in euros"]
    bet: f64,
) -> Result<(), Error> {

    let has_won_opt = ctx.data().coin_flip(ctx.author().id.get(), choice.to_lowercase().as_str(), bet).await;

    match has_won_opt {
        Some(true) => { ctx.say(format!("Congratulations, the coin landed on {}!\nYou won {} euros :euro:", choice.to_lowercase(), bet)).await.unwrap(); },
        Some(false) => { ctx.say(format!("Ups, you lost {} euros", bet)).await.unwrap(); },
        None => { ctx.say("Could not finish action").await.unwrap(); },
    }

    Ok(())
}

/// Claim daily reward.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn daily(
    ctx: Context<'_>,
) -> Result<(), Error> {

    match ctx.data().daily(ctx.author().id.get()).await {
        Some(true) => ctx.say(format!("Claimed daily reward `{}` euros", ctx.data().daily_amount)).await,
        Some(false) => ctx.say("Already claimed reward today").await,
        None => ctx.say("Could not finish action").await,
    }.unwrap();
    
    Ok(())
}