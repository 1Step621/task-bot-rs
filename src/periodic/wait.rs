use chrono::{Duration, Local, NaiveTime};
use poise::serenity_prelude::*;
use tokio::time::{sleep_until, Instant};

use crate::periodic::{backup, ping};

pub async fn wait(ctx: Context) {
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
        let sleep_duration = target_time - now;

        println!("Now: {}", now);
        println!("Next run: {}", target_time);
        println!("Sleeping for {} seconds", sleep_duration.num_seconds());

        sleep_until(Instant::now() + sleep_duration.to_std().unwrap()).await;
        ping::ping(&ctx).await.expect("Failed to ping");
        backup::backup(&ctx).await.expect("Failed to backup");
    }
}
