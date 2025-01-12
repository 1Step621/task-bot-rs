use std::iter;

use anyhow::{Context as _, Error};
use chrono::{Duration, NaiveTime};
use futures::StreamExt;
use poise::serenity_prelude::*;

use crate::{utilities::ResponsiveInteraction, PoiseContext};

pub async fn select_time(
    ctx: PoiseContext<'_>,
    interaction: Option<ResponsiveInteraction>,
    embed: Option<CreateEmbed>,
) -> Result<(ResponsiveInteraction, NaiveTime), Error> {
    const HOUR: &str = "hour";
    const MINUTE: &str = "minute";
    const SUBMIT: &str = "submit";

    let components = |selected_hour: Option<u32>, selected_minute: Option<u32>| {
        let hour_options = CreateSelectMenuKind::String {
            options: (0..24)
                .map(|i| {
                    CreateSelectMenuOption::new(i.to_string(), i.to_string())
                        .default_selection(selected_hour == Some(i))
                })
                .collect(),
        };
        let minute_options = CreateSelectMenuKind::String {
            options: (0..60)
                .step_by(5)
                .chain(iter::once(59))
                .map(|i| {
                    CreateSelectMenuOption::new(i.to_string(), i.to_string())
                        .default_selection(selected_minute == Some(i))
                })
                .collect(),
        };

        vec![
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new(HOUR, hour_options).placeholder("時"),
            ),
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new(MINUTE, minute_options).placeholder("分"),
            ),
            CreateActionRow::Buttons(vec![CreateButton::new(SUBMIT)
                .style(ButtonStyle::Primary)
                .label("送信")
                .disabled(selected_hour.is_none() || selected_minute.is_none())]),
        ]
    };

    let mut hour = None;
    let mut minute = None;

    let message = if let Some(interaction) = interaction {
        let response = CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::default().components(components(hour, minute)),
        );
        interaction.clone().create_response(ctx, response).await?;
        interaction.get_response(ctx).await?
    } else {
        ctx.send(
            if let Some(embed) = embed {
                poise::CreateReply::default().embed(embed)
            } else {
                poise::CreateReply::default()
            }
            .components(components(hour, minute)),
        )
        .await?
        .into_message()
        .await?
    };

    let mut interaction_stream = message
        .await_component_interaction(ctx)
        .timeout(Duration::seconds(60 * 30).to_std()?)
        .stream();

    let mut last_interaction = None;
    while let Some(interaction) = interaction_stream.next().await {
        match &interaction.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => {
                match interaction.data.custom_id.as_str() {
                    HOUR => {
                        hour.replace(values[0].parse().unwrap());
                        let response = CreateInteractionResponse::UpdateMessage(
                            CreateInteractionResponseMessage::default()
                                .components(components(hour, minute)),
                        );
                        interaction.create_response(ctx, response).await?;
                    }
                    MINUTE => {
                        minute.replace(values[0].parse().unwrap());
                        let response = CreateInteractionResponse::UpdateMessage(
                            CreateInteractionResponseMessage::default()
                                .components(components(hour, minute)),
                        );
                        interaction.create_response(ctx, response).await?;
                    }
                    _ => unreachable!(),
                }
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

    Ok((
        ResponsiveInteraction::Component(last_interaction.context("No interaction")?),
        NaiveTime::from_hms_opt(
            hour.context("Hour not selected")?,
            minute.context("Minute not selected")?,
            0,
        )
        .context("Invalid datetime")?,
    ))
}
