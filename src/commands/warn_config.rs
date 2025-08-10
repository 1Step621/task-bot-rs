use anyhow::Error;
use poise::serenity_prelude::*;

use crate::{PoiseContext, data};

#[poise::command(slash_command, dm_only)]
/// 宿題の期限接近通知を有効にします。DMで実行してください。
pub async fn enable_warn(ctx: PoiseContext<'_>) -> Result<(), Error> {
    ctx.data()
        .warn_users
        .lock()
        .unwrap()
        .insert(ctx.author().id);
    data::save(ctx.data())?;

    ctx.send(
        poise::CreateReply::default().embed(
            CreateEmbed::default()
                .title("期限接近通知を有効にしました")
                .color(Color::DARK_BLUE),
        ),
    )
    .await?;

    Ok(())
}

#[poise::command(slash_command, dm_only)]
/// 宿題の期限接近通知を無効にします。DMで実行してください。
pub async fn disable_warn(ctx: PoiseContext<'_>) -> Result<(), Error> {
    ctx.data()
        .warn_users
        .lock()
        .unwrap()
        .remove(&ctx.author().id);
    data::save(ctx.data())?;

    ctx.send(
        poise::CreateReply::default().embed(
            CreateEmbed::default()
                .title("期限接近通知を無効にしました")
                .color(Color::DARK_BLUE),
        ),
    )
    .await?;

    Ok(())
}
