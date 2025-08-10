use chrono::{Duration, Local, NaiveTime, Timelike};
use poise::serenity_prelude::*;
use tokio::time::{Instant, sleep_until};

use crate::periodic::{backup, ping, warn};

pub async fn every_day(ctx: Context) {
    loop {
        let now = Local::now();
        let target_time = {
            let time = Local::now()
                .with_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap())
                .unwrap();
            if time < now {
                time + Duration::days(1)
            } else {
                time
            }
        };
        println!("[every_day] Next execution at {}", target_time);
        let sleep_duration = target_time - now;

        sleep_until(Instant::now() + sleep_duration.to_std().unwrap()).await;
        ping::ping(&ctx).await.expect("Failed to ping");
        backup::backup(&ctx).await.expect("Failed to backup");
    }
}

pub async fn every_minute(ctx: Context) {
    loop {
        let now = Local::now();
        let target_time = {
            let time = Local::now()
                .with_second(0)
                .and_then(|t| t.with_nanosecond(0))
                .unwrap();
            if time < now {
                time + Duration::minutes(1)
            } else {
                time
            }
        };
        println!("[every_minute] Next execution at {}", target_time);
        let sleep_duration = target_time - now;

        sleep_until(Instant::now() + sleep_duration.to_std().unwrap()).await;
        warn::warn(&ctx).await.expect("Failed to warn");
    }
}
