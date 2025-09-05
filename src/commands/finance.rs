use poise::serenity_prelude::{Mention, User, UserId};
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

/// Displays the current price for a specific coin.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn price(
    ctx: Context<'_>,
) -> Result<(), Error> {
    Ok(())
}

/// Displays list of owned coins amount, the profit percentage, and absolute profit in euros.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn portfolio(
    ctx: Context<'_>,
) -> Result<(), Error> {
    Ok(())
}

/// Buy crypto currency in euros, if successful prints amount of coins bought.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn buy(
    ctx: Context<'_>,
) -> Result<(), Error> {
    Ok(())
}

/// Buy crypto currency in euros, if successful prints amount of coins bought.
#[poise::command(prefix_command, slash_command, category = "Finance")]
pub async fn sell(
    ctx: Context<'_>,
) -> Result<(), Error> {
    Ok(())
}