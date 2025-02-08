use anyhow::Error;

use poise::serenity_prelude::*;

use crate::{data, PoiseContext};

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
/// 管理者向けログを送るチャンネルを設定します。
pub async fn set_log_channel(
    ctx: PoiseContext<'_>,
    #[description = "管理者向けログを送るチャンネル"] channel: Option<Channel>,
) -> Result<(), Error> {
    let channel_id = channel.map(|c| c.id()).unwrap_or(ctx.channel_id());

    ctx.data()
        .log_channel
        .lock()
        .unwrap()
        .replace(channel_id);
    data::save(ctx.data())?;

    ctx.send(
        poise::CreateReply::default().embed(
            CreateEmbed::default()
                .title("ログチャンネルを設定しました")
                .description(format!("{}", channel_id.mention()))
                .color(Color::DARK_BLUE),
        ),
    )
    .await?;

    Ok(())
}
