#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {
    Mod,
    Admin,
}

pub async fn is_admin(ctx: crate::Context<'_>) -> Result<bool, crate::Error> {
    Ok(ctx.data().user_has_permission(ctx.author().id.get(), Permission::Admin))
}

pub async fn is_mod(ctx: crate::Context<'_>) -> Result<bool, crate::Error> {
    Ok(ctx.data().user_has_permission(ctx.author().id.get(), Permission::Mod))
}