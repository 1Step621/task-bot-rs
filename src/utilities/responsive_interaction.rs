use poise::serenity_prelude::*;

use crate::PoiseContext;

#[derive(Clone)]
pub enum ResponsiveInteraction {
    Component(ComponentInteraction),
    Modal(ModalInteraction),
}

impl ResponsiveInteraction {
    pub async fn create_response(
        &self,
        ctx: PoiseContext<'_>,
        response: CreateInteractionResponse,
    ) -> Result<(), Error> {
        match self {
            ResponsiveInteraction::Component(interaction) => {
                interaction.create_response(ctx, response).await
            }
            ResponsiveInteraction::Modal(interaction) => {
                interaction.create_response(ctx, response).await
            }
        }
    }

    pub async fn get_response(&self, ctx: PoiseContext<'_>) -> Result<Message, Error> {
        match self {
            ResponsiveInteraction::Component(interaction) => interaction.get_response(ctx).await,
            ResponsiveInteraction::Modal(interaction) => interaction.get_response(ctx).await,
        }
    }

    pub fn unwrap_component(self) -> ComponentInteraction {
        match self {
            ResponsiveInteraction::Component(interaction) => interaction,
            _ => panic!("Expected ComponentInteraction"),
        }
    }
}
