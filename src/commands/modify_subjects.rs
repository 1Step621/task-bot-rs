use std::time::Duration;

use anyhow::{Context as _, Error};
use futures::StreamExt;
use poise::serenity_prelude::*;

use crate::{data, PoiseContext};

#[poise::command(slash_command)]
/// 教科を追加します。
pub async fn add_subjects(
    ctx: PoiseContext<'_>,
    #[description = "追加したい教科 / カンマ区切りで複数追加できます"] subjects: String,
) -> Result<(), Error> {
    let subjects = subjects
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();

    ctx.data()
        .subjects
        .lock()
        .unwrap()
        .extend(subjects.clone().into_iter());
    data::save(ctx.data())?;

    let diff = format!(
        "```diff\n{}\n```",
        ctx.data()
            .subjects
            .lock()
            .unwrap()
            .iter()
            .map(|s| format!("{}{}", if subjects.contains(s) { "+ " } else { "" }, s))
            .collect::<Vec<_>>()
            .join("\n")
    );

    ctx.send(
        poise::CreateReply::default().embed(
            CreateEmbed::default()
                .title("追加しました")
                .description(diff)
                .color(Color::DARK_GREEN),
        ),
    )
    .await?;

    Ok(())
}

#[poise::command(slash_command)]
/// 教科を削除します。
pub async fn remove_subject(ctx: PoiseContext<'_>) -> Result<(), Error> {
    const SUBJECT: &str = "subject";
    const SUBMIT: &str = "submit";

    let subjects = ctx.data().subjects.lock().unwrap().clone();

    let components = |selected_subject: Option<String>| {
        let subject_options = CreateSelectMenuKind::String {
            options: subjects
                .iter()
                .map(|s| {
                    CreateSelectMenuOption::new(s, s)
                        .default_selection(selected_subject == Some(s.clone()))
                })
                .collect(),
        };

        vec![
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new(SUBJECT, subject_options)
                    .placeholder("削除したい教科を選択してください"),
            ),
            CreateActionRow::Buttons(vec![CreateButton::new(SUBMIT)
                .label("送信")
                .disabled(selected_subject.is_none())]),
        ]
    };

    let mut select = None;

    let message = ctx
        .send(
            poise::CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .title("削除したい教科を選択してください")
                        .color(Color::DARK_BLUE),
                )
                .components(components(select.clone())),
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
            ComponentInteractionDataKind::StringSelect { values, .. } => {
                if interaction.data.custom_id == SUBJECT {
                    select.replace(values[0].clone());
                }
                let response = CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::default()
                        .components(components(select.clone())),
                );
                interaction.create_response(&ctx, response).await?;
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

    let subject = select.context("Subject not selected")?;
    let diff = format!(
        "```diff\n{}\n```",
        ctx.data()
            .subjects
            .lock()
            .unwrap()
            .iter()
            .map(|s| format!("{}{}", if s == &subject { "- " } else { "" }, s))
            .collect::<Vec<_>>()
            .join("\n")
    );

    ctx.data()
        .subjects
        .lock()
        .unwrap()
        .retain(|s| s != &subject);
    data::save(ctx.data())?;

    let response = CreateInteractionResponse::UpdateMessage(
        CreateInteractionResponseMessage::default()
            .embed(
                CreateEmbed::default()
                    .title("削除しました")
                    .description(diff)
                    .color(Color::DARK_GREEN),
            )
            .components(vec![]),
    );

    last_interaction
        .context("No interaction")?
        .create_response(&ctx, response)
        .await?;

    Ok(())
}
