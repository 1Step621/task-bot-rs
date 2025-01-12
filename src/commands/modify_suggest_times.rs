use std::time::Duration;

use anyhow::{Context as _, Error};

use chrono::NaiveTime;
use futures::StreamExt;
use poise::serenity_prelude::*;

use crate::{data, interactions::select_time, PoiseContext};

#[poise::command(slash_command)]
/// よく使う時間を追加します。
pub async fn add_suggest_time(
    ctx: PoiseContext<'_>,
    #[description = "よく使う時間のラベル(例: 1限開始時刻)"] label: String,
) -> Result<(), Error> {
    let (interaction, time) = select_time(
        ctx,
        None,
        Some(
            CreateEmbed::default()
                .title(format!("よく使う時間({})を追加", label))
                .color(Color::DARK_BLUE),
        ),
    )
    .await?;

    ctx.data()
        .suggest_times
        .lock()
        .unwrap()
        .insert(time, label.clone());
    data::save(ctx.data())?;

    let title = format!("{}({})を追加しました", label, time.format("%H:%M"));
    let diff = format!(
        "```diff\n{}\n```",
        ctx.data()
            .suggest_times
            .lock()
            .unwrap()
            .iter()
            .map(|(t, l)| format!(
                "{}{}: {}",
                if l == &label { "+ " } else { "" },
                l,
                t.format("%H:%M")
            ))
            .collect::<Vec<String>>()
            .join("\n")
    );

    let response = CreateInteractionResponse::UpdateMessage(
        CreateInteractionResponseMessage::default()
            .embed(
                CreateEmbed::default()
                    .title(title)
                    .description(diff)
                    .color(Color::DARK_GREEN),
            )
            .components(vec![]),
    );

    interaction.create_response(ctx, response).await?;

    Ok(())
}

#[poise::command(slash_command)]
/// よく使う時間を削除します。
pub async fn remove_suggest_time(ctx: PoiseContext<'_>) -> Result<(), Error> {
    const LABEL: &str = "label";
    const SUBMIT: &str = "submit";

    let suggest_times = ctx.data().suggest_times.lock().unwrap().clone();

    let components = |selected_time: Option<NaiveTime>| {
        let suggest_time_options = CreateSelectMenuKind::String {
            options: suggest_times
                .iter()
                .map(|(t, l)| {
                    CreateSelectMenuOption::new(
                        format!("{} ({})", l, t.format("%H:%M")),
                        serde_json::to_string(t).unwrap(),
                    )
                    .default_selection(selected_time == Some(*t))
                })
                .collect(),
        };

        vec![
            CreateActionRow::SelectMenu(CreateSelectMenu::new(LABEL, suggest_time_options)),
            CreateActionRow::Buttons(vec![CreateButton::new(SUBMIT)
                .label("送信")
                .disabled(selected_time.is_none())]),
        ]
    };

    let mut time = None;

    let message = ctx
        .send(
            poise::CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .title("よく使う時間を削除")
                        .color(Color::DARK_BLUE),
                )
                .components(components(time)),
        )
        .await?;

    let mut interaction_stream = message
        .clone()
        .into_message()
        .await?
        .await_component_interaction(ctx)
        .timeout(Duration::from_secs(60 * 30))
        .stream();

    let mut last_interaction = None;
    while let Some(interaction) = interaction_stream.next().await {
        match &interaction.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => {
                if interaction.data.custom_id == LABEL {
                    time.replace(serde_json::from_str(&values[0])?);
                }
                let response = CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::default().components(components(time)),
                );
                interaction.create_response(ctx, response).await?;
            }
            ComponentInteractionDataKind::Button => {
                if interaction.data.custom_id == SUBMIT {
                    last_interaction.replace(interaction);
                    break;
                }
            }
            _ => unreachable!(),
        }
    }

    let time = time.context("No time selected")?;
    let title = format!(
        "{}({})を削除しました",
        suggest_times[&time],
        time.format("%H:%M"),
    );
    let diff = format!(
        "```diff\n{}\n```",
        ctx.data()
            .suggest_times
            .lock()
            .unwrap()
            .iter()
            .map(|(t, l)| format!(
                "{}{}: {}",
                if t == &time { "- " } else { "" },
                l,
                t.format("%H:%M")
            ))
            .collect::<Vec<String>>()
            .join("\n")
    );

    ctx.data().suggest_times.lock().unwrap().remove(&time);
    data::save(ctx.data())?;

    let response = CreateInteractionResponse::UpdateMessage(
        CreateInteractionResponseMessage::default()
            .embed(
                CreateEmbed::default()
                    .title(title)
                    .description(diff)
                    .color(Color::DARK_GREEN),
            )
            .components(vec![]),
    );

    last_interaction
        .context("No interaction")?
        .create_response(ctx, response)
        .await?;

    Ok(())
}
