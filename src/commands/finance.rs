use poise::serenity_prelude::{User, UserId};

use crate::{Context, Error, commands::get_user_name, model::ModelError, permissions::*};

/// Displays current money balance (in euros). If bank account does not exist, create one.
#[poise::command(
    prefix_command,
    slash_command,
    category = "Finance",
    aliases("balance")
)]
pub async fn bank(ctx: Context<'_>) -> anyhow::Result<()> {
    let user_id = ctx.author().id;

    match ctx.data().balance(user_id.get()).await {
        Ok(balance) => {
            ctx.say(format!(
                "**{}** has `{}` euros",
                get_user_name(&ctx, user_id).await,
                balance
            ))
            .await?;
        }
        Err(ModelError::BankAccountNotFound(_)) => {
            if ctx.data().create_bank_account(user_id.get()).await.is_ok() {
                ctx.say(format!(
                    "__Created bank account!__\n**{}** - `{}` (eur)",
                    get_user_name(&ctx, user_id).await,
                    0
                ))
                .await?;
            }
        }
        _ => Err(ModelError::UnexpectedError)?,
    }

    Ok(())
}

/// Give money (in euros) to another user.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn give(ctx: Context<'_>, dst_user: User, amount: f64) -> anyhow::Result<()> {
    let src_user_id = ctx.author().id.get();
    let dst_user_id = dst_user.id.get();

    match ctx.data().give(src_user_id, dst_user_id, amount).await {
        Ok((_, _)) => {
            ctx.say(format!(
                "{} gave {} `{}` euros.",
                ctx.author().name,
                dst_user.name,
                amount
            ))
            .await?;
        }
        Err(error @ ModelError::InsuficientFunds) | Err(error @ ModelError::InvalidValue(_)) => {
            ctx.say(error.to_string()).await?;
        }
        Err(ModelError::BankAccountNotFound(user_id)) => {
            ctx.say(format!(
                "User **{}** has no bank account",
                get_user_name(&ctx, UserId::new(user_id)).await
            ))
            .await?;
        }
        Err(error) => Err(error)?,
    }

    Ok(())
}

/// ADMIN COMMAND: Inject money (in euros) to another user.
#[poise::command(
    prefix_command,
    slash_command,
    owners_only,
    category = "Finance",
    check = "is_admin"
)]
pub async fn bless(ctx: Context<'_>, dst_user: User, amount: f64) -> anyhow::Result<()> {
    match ctx.data().bless(dst_user.id.get(), amount).await {
        Ok(_) => {
            ctx.say(format!(
                "**{}**, you were blessed with `{}` euros, amen :pray:",
                dst_user.name, amount
            ))
            .await?;
        }
        Err(ModelError::BankAccountNotFound(_)) => {
            ctx.say(format!("User **{}** has no bank account", dst_user.name))
                .await?;
        }
        Err(error) => Err(error)?,
    }

    Ok(())
}

/// Bank leaderboard. Who's the wealthiest.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    let bank_data = ctx.data().leaderboard().await?;

    if bank_data.is_empty() {
        ctx.say("No users in leaderboard").await?;
        return Ok(());
    }

    let mut output = String::new();
    for (user_id, balance) in bank_data {
        output.push_str(
            format!(
                "- **{}** has `{}` euros\n",
                get_user_name(&ctx, UserId::new(user_id)).await,
                balance
            )
            .as_str(),
        );
    }
    ctx.say(output).await?;

    Ok(())
}

/// Displays the current price for a specific coin. For more info go to https://coinmarketcap.com/
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn price(
    ctx: Context<'_>,
    #[description = "Crypto currency symbol (ie: btc, eth, ...)"] coin_symbol: String,
) -> Result<(), Error> {
    match ctx.data().coin_info(&coin_symbol).await {
        Ok(coin_info) => {
            ctx.say(format!(
                "**Name:** `{}`\n**Current Price:** `{}` euros",
                coin_info.name, coin_info.current_price
            ))
            .await?;
        }
        Err(error @ ModelError::InvalidValue(_)) => {
            ctx.say(error.to_string()).await?;
        }
        Err(error) => Err(error)?,
    }

    Ok(())
}

/// Displays list of owned coins amount, the profit percentage, and absolute profit in euros.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn portfolio(ctx: Context<'_>) -> Result<(), Error> {
    let portfolio_data = match ctx.data().portfolio(ctx.author().id.get()).await {
        Ok(portfolio_data) => portfolio_data,
        Err(ModelError::BankAccountNotFound(_)) => {
            ctx.say(format!(
                "User **{}** has no bank account",
                ctx.author().name
            ))
            .await?;
            return Ok(());
        }
        Err(error) => Err(error)?,
    };

    if portfolio_data.is_empty() {
        ctx.say("Empty portfolio!").await?;
        return Ok(());
    }

    let mut portfolio_str = "Portfolio:\n".to_string();
    for (coin_symbol, total_amount, total_value) in &portfolio_data {
        portfolio_str.push_str(
            format!(
                "- Symbol: **{}**, Total Amount: `{}` Total Value: `{}` euros\n",
                coin_symbol, total_amount, total_value
            )
            .as_str(),
        );
    }
    ctx.say(portfolio_str).await?;

    Ok(())
}

/// Buy crypto currency in euros, if successful prints amount of coins bought.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn buy(
    ctx: Context<'_>,
    #[description = "Crypto currency symbol (ie: btc, eth, ...)"] coin_symbol: String,
    #[description = "Value in euros of the amount of crypto you want to buy"] value: f64,
) -> Result<(), Error> {
    let amount_and_price_res = ctx
        .data()
        .buy(ctx.author().id.get(), &coin_symbol, value)
        .await;

    match amount_and_price_res {
        Ok((amount, price)) => {
            ctx.say(format!(
                "Successfully bought `{}` {} at `{}` euros",
                amount,
                coin_symbol.to_uppercase(),
                price
            ))
            .await?;
        }
        Err(ModelError::BankAccountNotFound(_)) => {
            ctx.say(format!(
                "User **{}** has no bank account",
                ctx.author().name
            ))
            .await?;
        }
        Err(error @ ModelError::InsuficientFunds) | Err(error @ ModelError::InvalidValue(_)) => {
            ctx.say(error.to_string()).await?;
        }
        Err(error) => Err(error)?,
    }

    Ok(())
}

/// Sell crypto currency in euros, if successful prints amount of coins bought.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn sell(
    ctx: Context<'_>,
    #[description = "Crypto currency symbol (ie: btc, eth, ...)"] coin_symbol: String,
    #[description = "Value in euros of the amount of crypto you want to sell"] value: f64,
) -> Result<(), Error> {
    let amount_and_price_res = ctx
        .data()
        .sell(ctx.author().id.get(), &coin_symbol, value)
        .await;

    match amount_and_price_res {
        Ok((amount, price)) => {
            ctx.say(format!(
                "Successfully sold `{}` {} at `{}` euros",
                amount,
                coin_symbol.to_uppercase(),
                price
            ))
            .await?;
        }
        Err(ModelError::BankAccountNotFound(_)) => {
            ctx.say(format!(
                "User **{}** has no bank account",
                ctx.author().name
            ))
            .await?;
        }
        Err(error @ ModelError::InvalidValue(_)) | Err(error @ ModelError::InsuficientCoins) => {
            ctx.say(error.to_string()).await?;
        }
        Err(error) => Err(error)?,
    }

    Ok(())
}

/// Sell crypto currency in euros, if successful prints amount of coins bought.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn sellall(
    ctx: Context<'_>,
    #[description = "Crypto currency symbol (ie: btc, eth, ...)"] coin_symbol: String,
) -> Result<(), Error> {
    let amount_and_price_res = ctx
        .data()
        .sell_all(ctx.author().id.get(), &coin_symbol)
        .await;

    match amount_and_price_res {
        Ok((amount, price)) => {
            ctx.say(format!(
                "Successfully sold `{}` {} at `{}` euros",
                amount,
                coin_symbol.to_uppercase(),
                price
            ))
            .await?;
        }
        Err(ModelError::BankAccountNotFound(_)) => {
            ctx.say(format!(
                "User **{}** has no bank account",
                ctx.author().name
            ))
            .await?;
        }
        Err(error @ ModelError::InsuficientCoins) => {
            ctx.say(error.to_string()).await?;
        }
        Err(error) => Err(error)?,
    }

    Ok(())
}

/// Bet on heads or tails.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn coin(
    ctx: Context<'_>,
    #[description = "Heads or tails"] choice: String,
    #[description = "Bet amount in euros"] bet: f64,
) -> Result<(), Error> {
    let has_won_res = ctx
        .data()
        .coin_flip(ctx.author().id.get(), choice.to_lowercase().as_str(), bet)
        .await;

    match has_won_res {
        Ok(true) => {
            ctx.say(format!(
                "Congratulations, the coin landed on {}!\nYou won {} euros :euro:",
                choice.to_lowercase(),
                bet
            ))
            .await?;
        }
        Ok(false) => {
            ctx.say(format!("Ups, you lost {} euros", bet)).await?;
        }
        Err(error @ ModelError::InvalidValue(_)) => {
            ctx.say(error.to_string()).await?;
        }
        Err(ModelError::BankAccountNotFound(_)) => {
            ctx.say(format!(
                "User **{}** has no bank account",
                ctx.author().name
            ))
            .await?;
        }
        Err(error) => Err(error)?,
    }

    Ok(())
}

/// Claim daily reward.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn daily(ctx: Context<'_>) -> Result<(), Error> {
    match ctx.data().daily(ctx.author().id.get()).await {
        Ok(true) => {
            ctx.say(format!(
                "Claimed daily reward `{}` euros",
                ctx.data().daily_amount
            ))
            .await?;
        }
        Ok(false) => {
            ctx.say("Already claimed reward today").await?;
        }
        Err(ModelError::BankAccountNotFound(_)) => {
            ctx.say(format!(
                "User **{}** has no bank account",
                ctx.author().name
            ))
            .await?;
        }
        Err(error) => Err(error)?,
    }

    Ok(())
}
