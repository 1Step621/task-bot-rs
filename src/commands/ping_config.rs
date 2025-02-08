use anyhow::{Context as _, Error};
use chrono::{Local, TimeZone};
use poise::serenity_prelude::*;

use crate::{data, interactions::select_date, PoiseContext};

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
/// タスク通知を送るチャンネルを設定します。
pub async fn set_ping_channel(
    ctx: PoiseContext<'_>,
    #[description = "タスク通知を送るチャンネル"] channel: Option<Channel>,
) -> Result<(), Error> {
    let channel_id = channel.map(|c| c.id()).unwrap_or(ctx.channel_id());

    ctx.data()
        .ping_channel
        .lock()
        .unwrap()
        .replace(channel_id);
    data::save(ctx.data())?;

    ctx.send(
        poise::CreateReply::default().embed(
            CreateEmbed::default()
                .title("通知チャンネルを設定しました")
                .description(format!("{}", channel_id.mention()))
                .color(Color::DARK_BLUE),
        ),
    )
    .await?;

    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
/// タスク通知を送るロールを設定します。
pub async fn set_ping_role(
    ctx: PoiseContext<'_>,
    #[description = "タスク通知を送るロール"] role: Role,
) -> Result<(), Error> {
    ctx.data().ping_role.lock().unwrap().replace(role.id);
    data::save(ctx.data())?;

    ctx.send(
        poise::CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .title("ロールを設定しました")
                    .description(format!("{}", role.id.mention()))
                    .color(Color::DARK_BLUE),
            )
            .components(vec![]),
    )
    .await?;

    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
/// タスク通知をある日付まで停止します。
pub async fn stop_ping(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let (last_interaction, date) = select_date(
        ctx,
        None,
        Some(
            CreateEmbed::default()
                .title("何日まで停止するかを選択してください")
                .description("その日から通知を再開します")
                .color(Color::DARK_BLUE),
        ),
    )
    .await
    .context("No date selected")?;

    let date = Local
        .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
        .unwrap();
    let timestamp = date.timestamp();

    *ctx.data().stop_ping_until.lock().unwrap() = date;
    data::save(ctx.data())?;

    last_interaction
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::default()
                    .embed(
                        CreateEmbed::default()
                            .title(format!(
                                "<t:{}:D>(<t:{}:R>)まで通知を停止します",
                                timestamp, timestamp
                            ))
                            .color(Color::DARK_BLUE),
                    )
                    .components(vec![]),
            ),
        )
        .await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
/// 通知の停止を解除します。
pub async fn resume_ping(ctx: PoiseContext<'_>) -> Result<(), Error> {
    *ctx.data().stop_ping_until.lock().unwrap() = Local::now();
    data::save(ctx.data())?;

    ctx.send(
        poise::CreateReply::default().embed(
            CreateEmbed::default()
                .title("通知を再開しました")
                .color(Color::DARK_BLUE),
        ),
    )
    .await?;
    Ok(())
}
