use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {
    Mod,
    Admin,
}

async fn has_permission(
    ctx: crate::Context<'_>,
    permission: Permission,
) -> Result<bool, crate::Error> {
    let has_permission = ctx
        .data()
        .user_has_permission(ctx.author().id.get(), permission);
    if !has_permission {
        warn!(
            "User {} does not have {:?} permission",
            ctx.author().name,
            permission
        );
        ctx.say(format!(
            "User {} does not have permission to use this command",
            ctx.author().name
        ))
        .await?;
    }

    Ok(has_permission)
}

pub async fn is_admin(ctx: crate::Context<'_>) -> Result<bool, crate::Error> {
    has_permission(ctx, Permission::Admin).await
}

pub async fn is_mod(ctx: crate::Context<'_>) -> Result<bool, crate::Error> {
    has_permission(ctx, Permission::Admin).await
}
