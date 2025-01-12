use anyhow::{Context, Error};
use chrono::Duration;
use poise::serenity_prelude::*;

use crate::{utilities::ResponsiveInteraction, PoiseContext};

pub async fn select_announce(
    ctx: PoiseContext<'_>,
    interaction: Option<ResponsiveInteraction>,
) -> Result<(ResponsiveInteraction, bool), Error> {
    const ANNOUNCE: &str = "announce";
    const CANCEL: &str = "cancel";

    let embed = CreateEmbed::default()
        .title("課題通知メッセージを更新しました")
        .description("更新をアナウンスしますか？")
        .color(Color::DARK_BLUE);
    let components = vec![CreateActionRow::Buttons(vec![
        CreateButton::new("announce")
            .style(ButtonStyle::Primary)
            .label("アナウンスする")
            .custom_id(ANNOUNCE),
        CreateButton::new("cancel")
            .style(ButtonStyle::Danger)
            .label("アナウンスしない")
            .custom_id(CANCEL),
    ])];

    let message = if let Some(interaction) = interaction {
        let response = CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::default()
                .embed(embed)
                .components(components),
        );
        interaction.create_response(ctx, response).await?;
        interaction.get_response(ctx).await?
    } else {
        ctx.send(
            poise::CreateReply::default()
                .embed(embed)
                .components(components),
        )
        .await?
        .into_message()
        .await?
    };

    let interaction = message
        .await_component_interaction(ctx)
        .timeout(Duration::days(7).to_std()?)
        .await
        .context("No interaction")?;

    match &interaction.data.kind {
        ComponentInteractionDataKind::Button => match interaction.data.custom_id.as_str() {
            ANNOUNCE => Ok((ResponsiveInteraction::Component(interaction), true)),
            CANCEL => Ok((ResponsiveInteraction::Component(interaction), false)),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}
