use anyhow::Error;

use poise::serenity_prelude::*;

use crate::{data, PoiseContext};

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
/// 管理者向けログを送るチャンネルを設定します。
pub async fn set_log_channel(ctx: PoiseContext<'_>) -> Result<(), Error> {
    ctx.data()
        .log_channel
        .lock()
        .unwrap()
        .replace(ctx.channel_id());
    data::save(ctx.data())?;

    ctx.send(
        poise::CreateReply::default().embed(
            CreateEmbed::default()
                .title("ログチャンネルを設定しました")
                .description(format!("{}", ctx.channel_id().mention()))
                .color(Color::DARK_BLUE),
        ),
    )
    .await?;

    Ok(())
}
