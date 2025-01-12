use anyhow::{Context as _, Error};
use chrono::{Datelike, Duration, Local, NaiveDate};
use futures::StreamExt;
use poise::serenity_prelude::*;
use serde::{Deserialize, Serialize};

use crate::{utilities::ResponsiveInteraction, PoiseContext};

#[derive(Serialize, Deserialize, Clone, Copy)]
struct MonthHalf {
    month: u32,
    is_first_half: bool,
}

impl From<MonthHalf> for String {
    fn from(e: MonthHalf) -> Self {
        format!(
            "{}月{}",
            e.month,
            if e.is_first_half {
                "前半(〜15)"
            } else {
                "後半(16〜)"
            }
        )
    }
}

pub async fn select_date(
    ctx: PoiseContext<'_>,
    interaction: Option<ResponsiveInteraction>,
    embed: Option<CreateEmbed>,
) -> Result<(ResponsiveInteraction, NaiveDate), Error> {
    const YEAR: &str = "year";
    const MONTH: &str = "month";
    const DAY: &str = "day";
    const SUBMIT: &str = "submit";

    let mut last_interaction = None;
    let mut date = Local::now().date_naive();

    let components = |date: NaiveDate| {
        let month = date.month();
        let is_first_half = date.day() <= 15;

        let year_options = CreateSelectMenuKind::String {
            options: (Local::now().year()..=Local::now().year() + 2)
                .map(|i| {
                    CreateSelectMenuOption::new(i.to_string(), i.to_string())
                        .default_selection(i == date.year())
                })
                .collect(),
        };
        let month_options = CreateSelectMenuKind::String {
            options: (1..=12)
                .flat_map(|i| {
                    [
                        MonthHalf {
                            month: i,
                            is_first_half: true,
                        },
                        MonthHalf {
                            month: i,
                            is_first_half: false,
                        },
                    ]
                })
                .map(|e| {
                    CreateSelectMenuOption::new(String::from(e), serde_json::to_string(&e).unwrap())
                        .default_selection(month == e.month && is_first_half == e.is_first_half)
                })
                .collect(),
        };
        let day_options = CreateSelectMenuKind::String {
            options: if is_first_half {
                1..=15
            } else {
                16..=days_in_month(date.year(), month).unwrap()
            }
            .map(|i| {
                CreateSelectMenuOption::new(i.to_string(), i.to_string())
                    .default_selection(i == date.day())
            })
            .collect(),
        };

        vec![
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new(YEAR, year_options).placeholder("年"),
            ),
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new(MONTH, month_options).placeholder("月"),
            ),
            CreateActionRow::SelectMenu(CreateSelectMenu::new(DAY, day_options).placeholder("日")),
            CreateActionRow::Buttons(vec![CreateButton::new(SUBMIT)
                .style(ButtonStyle::Primary)
                .label("送信")]),
        ]
    };

    let message = if let Some(interaction) = interaction {
        let response = CreateInteractionResponse::UpdateMessage(
            if let Some(embed) = embed {
                CreateInteractionResponseMessage::default().embed(embed)
            } else {
                CreateInteractionResponseMessage::default()
            }
            .components(components(date)),
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
            .components(components(date)),
        )
        .await?
        .into_message()
        .await?
    };

    let mut interaction_stream = message
        .await_component_interaction(ctx)
        .timeout(Duration::seconds(60 * 30).to_std()?)
        .stream();

    fn days_in_month(year: i32, month: u32) -> Result<u32, Error> {
        // 次の月の1日から1日引くことで、その月の最終日を取得
        let next_month = if month == 12 { 1 } else { month + 1 };
        let next_year = if month == 12 { year + 1 } else { year };

        let last_day = NaiveDate::from_ymd_opt(next_year, next_month, 1)
            .context("Invalid date")?
            .pred_opt()
            .context("Invalid date")?;

        Ok(last_day.day())
    }

    while let Some(interaction) = interaction_stream.next().await {
        match &interaction.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => {
                match interaction.data.custom_id.as_str() {
                    YEAR => {
                        date = date
                            .with_year(values[0].parse().unwrap())
                            .context("Invalid date")?;
                        interaction
                            .create_response(ctx, CreateInteractionResponse::Acknowledge)
                            .await?;
                    }
                    MONTH => {
                        let selected_month: MonthHalf = serde_json::from_str(&values[0])?;
                        date = date
                            .with_month(selected_month.month)
                            .and_then(|d| {
                                d.with_day(if selected_month.is_first_half { 1 } else { 16 })
                            })
                            .context("Invalid date")?;

                        let response = CreateInteractionResponse::UpdateMessage(
                            CreateInteractionResponseMessage::default()
                                .components(components(date)),
                        );
                        interaction.create_response(ctx, response).await?;
                    }
                    DAY => {
                        date = date
                            .with_day(values[0].parse().unwrap())
                            .context("Invalid date")?;
                        interaction
                            .create_response(ctx, CreateInteractionResponse::Acknowledge)
                            .await?;
                    }
                    _ => {}
                }
            }
            ComponentInteractionDataKind::Button => {
                if interaction.data.custom_id == SUBMIT {
                    last_interaction.replace(interaction);
                    break;
                }
            }
            _ => {}
        }
    }

    Ok((ResponsiveInteraction::Component(last_interaction.context("No interaction")?), date))
}
