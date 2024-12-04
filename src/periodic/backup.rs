use anyhow::{Context as _, Error};
use chrono::Local;
use poise::serenity_prelude::*;
use tokio::fs::File;

use crate::{data, utilities::format_datetime};

pub async fn backup(ctx: &Context) -> Result<(), Error> {
    let data = data::load()?;
    let log_channel = (*data.log_channel.lock().unwrap()).context("Log channel not set")?;

    log_channel
        .send_files(
            ctx,
            vec![
                CreateAttachment::file(
                    &File::open(data::FILE_PATH).await?,
                    format!("{}.json", Local::now().timestamp()),
                )
                .await?,
            ],
            CreateMessage::default().embed(CreateEmbed::default().title(format!(
                "データのバックアップ ({})",
                format_datetime(Local::now())
            ))),
        )
        .await?;

    Ok(())
}
