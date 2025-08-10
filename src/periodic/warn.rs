use anyhow::Error;
use chrono::{DateTime, Duration, Local, Timelike};
use itertools::Itertools;
use poise::serenity_prelude::*;

use crate::{Category, Task, data};

fn search_tasks(from: DateTime<Local>, to: DateTime<Local>) -> Result<Vec<Task>, Error> {
    let data = data::load()?;

    let tasks = data.tasks.lock().unwrap().clone();

    Ok(tasks
        .iter()
        .filter(|task| task.category == Category::Homework)
        .filter(|task| from <= task.datetime && task.datetime < to)
        .sorted_by_key(|task| task.datetime)
        .cloned()
        .collect())
}

fn period(now: DateTime<Local>) -> (DateTime<Local>, DateTime<Local>) {
    let from = (now + Duration::hours(1))
        .with_second(0)
        .and_then(|t| t.with_nanosecond(0))
        .unwrap();
    let to = (now + Duration::hours(1) + Duration::minutes(1))
        .with_second(0)
        .and_then(|t| t.with_nanosecond(0))
        .unwrap();

    (from, to)
}

fn embed(tasks: &Vec<Task>) -> CreateEmbed {
    let fields = tasks.iter().map(|task| task.to_field()).collect::<Vec<_>>();

    CreateEmbed::default()
        .title("宿題の期限が接近しています")
        .fields(fields)
        .color(Color::RED)
}

pub async fn warn(ctx: &Context) -> Result<(), Error> {
    let data = data::load()?;

    let (from, to) = period(Local::now());

    println!("Searching for homework tasks from {} to {}", from, to);

    let tasks = search_tasks(from, to)?;
    if tasks.is_empty() {
        println!("No homework tasks to warn about.");
        return Ok(());
    }

    let warn_users = data.warn_users.lock().unwrap().clone();

    for user in warn_users {
        user.direct_message(ctx, CreateMessage::default().embed(embed(&tasks)))
            .await?;
    }

    Ok(())
}
