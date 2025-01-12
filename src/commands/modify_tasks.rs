use anyhow::{Context, Error};
use poise::serenity_prelude::*;

use crate::{
    data,
    interactions::{create_task, select_announce, select_task},
    periodic::ping,
    PartialTask, PoiseContext,
};

#[poise::command(slash_command)]
/// タスクを追加します。
pub async fn add_task(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let ping_role = (*ctx.data().ping_role.lock().unwrap()).context("Ping role not set")?;

    let (mut last_interaction, task) = create_task(
        ctx,
        None,
        Some(
            CreateEmbed::default()
                .title("タスクを追加します".to_string())
                .color(Color::DARK_BLUE),
        ),
        PartialTask::default(),
    )
    .await?;

    ctx.data().tasks.lock().unwrap().insert(task.clone());
    data::save(ctx.data())?;

    let embed = CreateEmbed::default()
        .title("タスクを追加しました")
        .fields(vec![task.to_field()])
        .color(Color::DARK_GREEN);

    if let Some(message) = ping::update(&ctx).await?.last() {
        let announce;
        (last_interaction, announce) = select_announce(ctx, Some(last_interaction)).await?;
        if announce {
            message
                .channel_id
                .send_message(
                    ctx,
                    CreateMessage::default()
                        .content(format!(
                            "{}\nタスクが追加されました！ご注意ください！",
                            ping_role.mention()
                        ))
                        .embed(embed.clone())
                        .reference_message(message),
                )
                .await?;
        }
    }

    let response = CreateInteractionResponse::UpdateMessage(
        CreateInteractionResponseMessage::default()
            .embed(embed)
            .components(vec![]),
    );
    last_interaction.create_response(ctx, response).await?;

    Ok(())
}

#[poise::command(slash_command)]
/// タスクを削除します。
pub async fn remove_task(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let ping_role = (*ctx.data().ping_role.lock().unwrap()).context("Ping role not set")?;

    let (mut last_interaction, task) = select_task(
        ctx,
        None,
        Some(
            CreateEmbed::default()
                .title("削除するタスクを選択")
                .color(Color::DARK_BLUE),
        ),
    )
    .await?;

    {
        let mut tasks = ctx.data().tasks.lock().unwrap();
        tasks.remove(&task);
    }
    data::save(ctx.data())?;

    let embed = CreateEmbed::default()
        .title("削除しました")
        .fields(vec![task.to_field()])
        .color(Color::DARK_RED);

    if let Some(message) = ping::update(&ctx).await?.last() {
        let announce;
        (last_interaction, announce) = select_announce(ctx, Some(last_interaction)).await?;
        if announce {
            message
                .channel_id
                .send_message(
                    ctx,
                    CreateMessage::default()
                        .content(format!(
                            "{}\nタスクが削除されました！ご注意ください！",
                            ping_role.mention()
                        ))
                        .embed(embed.clone())
                        .reference_message(message),
                )
                .await?;
        }
    }

    let response = CreateInteractionResponse::UpdateMessage(
        CreateInteractionResponseMessage::default()
            .embed(embed)
            .components(vec![]),
    );
    last_interaction.create_response(ctx, response).await?;

    ping::update(&ctx).await?;

    Ok(())
}

#[poise::command(slash_command)]
/// タスクを編集します。
pub async fn edit_task(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let ping_role = (*ctx.data().ping_role.lock().unwrap()).context("Ping role not set")?;

    let (last_interaction, task) = select_task(
        ctx,
        None,
        Some(
            CreateEmbed::default()
                .title("編集するタスクを選択")
                .color(Color::DARK_BLUE),
        ),
    )
    .await?;

    let (mut last_interaction, modified_task) = create_task(
        ctx,
        Some(last_interaction),
        Some(
            CreateEmbed::default()
                .title("タスクを編集します".to_string())
                .color(Color::DARK_BLUE),
        ),
        task.as_partial(),
    )
    .await?;

    {
        let mut tasks = ctx.data().tasks.lock().unwrap();
        tasks.remove(&task);
        tasks.insert(modified_task.clone());
    }
    data::save(ctx.data())?;

    let embed = CreateEmbed::default()
        .title("タスクを編集しました")
        .fields(vec![
            task.to_field(),
            ("↓".into(), "".into(), false),
            modified_task.to_field(),
        ])
        .color(Color::DARK_GREEN);

    if let Some(message) = ping::update(&ctx).await?.last() {
        let announce;
        (last_interaction, announce) = select_announce(ctx, Some(last_interaction)).await?;
        if announce {
            message
                .channel_id
                .send_message(
                    ctx,
                    CreateMessage::default()
                        .content(format!(
                            "{}\nタスクが編集されました！ご注意ください！",
                            ping_role.mention()
                        ))
                        .embed(embed.clone())
                        .reference_message(message),
                )
                .await?;
        }
    }

    let response = CreateInteractionResponse::UpdateMessage(
        CreateInteractionResponseMessage::default()
            .embed(embed)
            .components(vec![]),
    );
    last_interaction.create_response(ctx, response).await?;

    ping::update(&ctx).await?;

    Ok(())
}
