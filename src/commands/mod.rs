use poise::serenity_prelude::UserId;

pub mod help;
pub use help::*;

pub mod white_monster;
pub use white_monster::*;

pub mod finance;
pub use finance::*;

pub async fn get_user_name(ctx: &crate::Context<'_>, user_id: UserId) -> String {
    if let Some(cached_user) = ctx.cache().user(user_id) {
        cached_user.display_name().into()
    } else {
        let user_res = ctx.http().get_user(user_id).await;
        if let Ok(user) = user_res {
            user.display_name().into()
        } else {
            "".into()
        }
    }
}
