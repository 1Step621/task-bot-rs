use std::iter;

use anyhow::{Context as _, Error};
use chrono::{Duration, Local, NaiveDate, NaiveTime};
use futures::StreamExt;
use poise::serenity_prelude::*;

use crate::{
    interactions::{select_date, select_time},
    utilities::{format_date, ResponsiveInteraction},
    Category, PartialTask, PoiseContext, Subject, Task,
};

pub async fn create_task(
    ctx: PoiseContext<'_>,
    interaction: Option<ResponsiveInteraction>,
    embed: Option<CreateEmbed>,
    defaults: PartialTask,
) -> Result<(ResponsiveInteraction, Task), Error> {
    const CATEGORY: &str = "category";
    const SUBJECT: &str = "subject";
    const DATE: &str = "date";
    const TIME: &str = "time";
    const SUBMIT: &str = "submit";

    let subjects = ctx.data().subjects.lock().unwrap().clone();
    let suggest_times = ctx.data().suggest_times.lock().unwrap().clone();

    let components = |task: &PartialTask| {
        let category_options = CreateSelectMenuKind::String {
            options: Category::VALUES
                .iter()
                .map(|&c| {
                    CreateSelectMenuOption::new(c, serde_json::to_string(&c).unwrap())
                        .default_selection(task.category == Some(c))
                })
                .collect(),
        };
        let subject_options = CreateSelectMenuKind::String {
            options: subjects
                .iter()
                .map(|s| {
                    CreateSelectMenuOption::new(
                        s,
                        serde_json::to_string(&Subject::Set(s.to_string())).unwrap(),
                    )
                    .default_selection(task.subject == Some(Subject::Set(s.to_string())))
                })
                .chain(iter::once(
                    CreateSelectMenuOption::new(
                        "(教科を指定しない)",
                        serde_json::to_string(&Subject::Unset).unwrap(),
                    )
                    .default_selection(task.subject == Some(Subject::Unset)),
                ))
                .collect(),
        };
        let date_options = CreateSelectMenuKind::String {
            options: (0..24)
                .map(|i| {
                    let date = Local::now().date_naive() + Duration::days(i);
                    CreateSelectMenuOption::new(
                        format_date(date),
                        serde_json::to_string(&Some(date)).unwrap(),
                    )
                    .default_selection(task.date == Some(date))
                })
                .chain(iter::once(
                    CreateSelectMenuOption::new(
                        "その他の日付",
                        serde_json::to_string(&None::<NaiveDate>).unwrap(),
                    )
                    .default_selection(task.date.is_none()),
                ))
                .collect(),
        };
        let time_options = CreateSelectMenuKind::String {
            options: suggest_times
                .iter()
                .map(|(t, l)| {
                    CreateSelectMenuOption::new(
                        format!("{} ({})", l, t.format("%H:%M")),
                        serde_json::to_string(&Some(t)).unwrap(),
                    )
                    .default_selection(task.time == Some(*t))
                })
                .chain(iter::once(
                    CreateSelectMenuOption::new(
                        "その他の時刻",
                        serde_json::to_string(&None::<NaiveTime>).unwrap(),
                    )
                    .default_selection(task.time.is_none()),
                ))
                .collect::<Vec<_>>(),
        };

        vec![
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new(CATEGORY, category_options).placeholder("カテゴリー"),
            ),
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new(SUBJECT, subject_options).placeholder("教科"),
            ),
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new(DATE, date_options)
                    .placeholder(task.date.map_or("日付".into(), format_date)),
            ),
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new(TIME, time_options).placeholder(
                    task.time
                        .map_or("時間".into(), |x| x.format("%H:%M").to_string()),
                ),
            ),
            CreateActionRow::Buttons(vec![CreateButton::new(SUBMIT)
                .style(ButtonStyle::Primary)
                .label("送信")
                .disabled(task.category.is_none() || task.subject.is_none())]),
        ]
    };

    let message = if let Some(interaction) = interaction {
        let response = CreateInteractionResponse::UpdateMessage(
            if let Some(embed) = embed {
                CreateInteractionResponseMessage::default().embed(embed)
            } else {
                CreateInteractionResponseMessage::default()
            }
            .components(components(&defaults)),
        );
        interaction.create_response(ctx, response).await?;
        interaction.get_response(ctx).await?
    } else {
        ctx.send(
            if let Some(embed) = embed {
                poise::CreateReply::default().embed(embed)
            } else {
                poise::CreateReply::default()
            }
            .components(components(&defaults)),
        )
        .await?
        .into_message()
        .await?
    };

    let mut interaction_stream = message
        .clone()
        .await_component_interaction(ctx)
        .timeout(Duration::seconds(60 * 30).to_std()?)
        .stream();

    let mut task = defaults.clone();

    let mut last_interaction = None;
    while let Some(interaction) = interaction_stream.next().await {
        match &interaction.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => {
                match interaction.data.custom_id.as_str() {
                    CATEGORY => {
                        task.category.replace(serde_json::from_str(&values[0])?);
                    }
                    SUBJECT => {
                        task.subject.replace(serde_json::from_str(&values[0])?);
                    }
                    DATE => {
                        task.date = serde_json::from_str(&values[0])?;
                    }
                    TIME => {
                        task.time = serde_json::from_str(&values[0])?;
                    }
                    _ => unreachable!(),
                }
                let response = CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::default().components(components(&task)),
                );
                interaction.create_response(&ctx, response).await?;
            }
            ComponentInteractionDataKind::Button => {
                if interaction.data.custom_id == SUBMIT {
                    last_interaction.replace(ResponsiveInteraction::Component(interaction));
                    break;
                }
            }
            _ => unreachable!(),
        }
    }

    task.date = match task.clone().date {
        Some(date) => Some(date),
        None => {
            let (interaction, date) = select_date(
                ctx,
                Some(last_interaction.clone().context("No interaction")?),
                None,
            )
            .await?;
            last_interaction.replace(interaction);
            Some(date)
        }
    };

    task.time = match task.clone().time {
        Some(time) => Some(time),
        None => {
            let (interaction, time) = select_time(
                ctx,
                Some(last_interaction.clone().context("No interaction")?),
                None,
            )
            .await?;
            last_interaction.replace(interaction);
            Some(time)
        }
    };

    let modal = CreateQuickModal::new("詳細入力")
        .field(
            CreateInputText::new(InputTextStyle::Short, "詳細", "")
                .value(task.details.unwrap_or("".into()))
                .placeholder("詳細を入力してください"),
        )
        .timeout(Duration::seconds(60 * 30).to_std()?);

    let response = last_interaction
        .clone()
        .context("No interaction")?
        .unwrap_component()
        .quick_modal(ctx.serenity_context(), modal)
        .await?;

    let QuickModalResponse {
        inputs,
        interaction,
    } = response.context("No response")?;

    task.details = Some(inputs[0].clone());

    let task = task.unpartial()?;

    Ok((ResponsiveInteraction::Modal(interaction), task))
}
