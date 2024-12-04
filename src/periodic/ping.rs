use anyhow::{Context as _, Error};
use chrono::{DateTime, Duration, Local, NaiveTime};
use itertools::Itertools;
use poise::serenity_prelude::*;

use crate::{data, PoiseContext, Task};

fn search_tasks(from: DateTime<Local>, to: DateTime<Local>) -> Result<Vec<Task>, Error> {
    let data = data::load()?;

    let tasks = data.tasks.lock().unwrap().clone();

    Ok(tasks
        .iter()
        .filter(|task| from < task.datetime && task.datetime <= to)
        .sorted_by_key(|task| task.datetime)
        .cloned()
        .collect())
}

fn embed(tasks: Vec<Task>) -> Result<CreateEmbed, Error> {
    let fields = tasks.iter().map(|task| task.to_field());

    Ok(if fields.len() > 0 {
        CreateEmbed::default()
            .title("タスク通知")
            .description("明日のタスクをお知らせします！")
            .fields(fields)
            .color(Color::RED)
    } else {
        CreateEmbed::default()
            .title("タスク通知")
            .description("明日のタスクはありません:tada:")
            .color(Color::DARK_GREEN)
    })
}

pub async fn ping(ctx: &Context) -> Result<(), Error> {
    let data = data::load()?;

    let ping_channel = (*data.ping_channel.lock().unwrap()).context("Ping channel not set")?;
    let ping_role = (*data.ping_role.lock().unwrap()).context("Ping role not set")?;

    let from = (Local::now() + Duration::days(1))
        .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .unwrap();
    let to = (Local::now() + Duration::days(2))
        .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .unwrap();

    println!("Searching tasks: from {} to {}", from, to);

    let tasks = search_tasks(from, to)?;
    ping_channel
        .send_message(
            ctx,
            if !tasks.is_empty() {
                CreateMessage::default().content(format!("{}", ping_role.mention()))
            } else {
                CreateMessage::default()
            }
            .embed(embed(tasks)?),
        )
        .await?;

    Ok(())
}

pub async fn update(ctx: &PoiseContext<'_>) -> Result<(), Error> {
    let data = data::load()?;

    let ping_channel = (*data.ping_channel.lock().unwrap()).context("Ping channel not set")?;
    let ping_role = (*data.ping_role.lock().unwrap()).context("Ping role not set")?;

    let from = (Local::now() + Duration::days(1))
        .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .unwrap();
    let to = (Local::now() + Duration::days(2))
        .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .unwrap();

    let prev_message = ping_channel
        .messages(ctx, GetMessages::default())
        .await?
        .into_iter()
        .sorted_by_key(|m| m.id.created_at())
        .rev()
        .find(|m| {
            m.author.id == ctx.framework().bot_id
                && m.id.created_at().date_naive() == Local::now().date_naive()
        });
    let Some(prev_message) = prev_message else {
        println!("No previous embed found; Updating not needed");
        return Ok(());
    };

    let prev_embed = prev_message.embeds[0].clone();

    if CreateEmbed::from(prev_embed) != embed(search_tasks(from, to)?)? {
        ping_channel
            .send_message(
                ctx,
                CreateMessage::default()
                    .reference_message(&prev_message)
                    .content(format!(
                        "{}\n更新があります！ご注意ください！",
                        ping_role.mention()
                    ))
                    .embed(embed(search_tasks(from, to)?)?),
            )
            .await?;
        println!("Message updated");
    } else {
        println!("No changes; Updating not needed");
    }

    Ok(())
}
