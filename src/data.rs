use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    fs,
    sync::Mutex,
};

use anyhow::{Context, Error};
use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone};
use poise::serenity_prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    // イベント
    Event,
    // テスト
    Exam,
    // 宿題
    Homework,
    // 持ち物
    Belongings,
    // その他
    Other,
}

impl From<Category> for String {
    fn from(category: Category) -> Self {
        match category {
            Category::Event => "イベント",
            Category::Exam => "テスト",
            Category::Homework => "宿題",
            Category::Belongings => "持ち物",
            Category::Other => "その他",
        }
        .to_string()
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&String::from(*self))
    }
}

impl Category {
    pub const VALUES: [Category; 5] = [
        Category::Event,
        Category::Exam,
        Category::Homework,
        Category::Belongings,
        Category::Other,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Subject {
    Set(String),
    Unset,
}

impl Serialize for Subject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Subject::Set(s) => serializer.serialize_str(s),
            Subject::Unset => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for Subject {
    fn deserialize<D>(deserializer: D) -> Result<Subject, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = Option::<String>::deserialize(deserializer)?;
        Ok(match s {
            Some(s) => Subject::Set(s),
            None => Subject::Unset,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Task {
    pub category: Category,
    pub subject: Subject,
    pub details: String,
    pub datetime: DateTime<Local>,
}

impl Task {
    pub fn to_field(&self) -> (String, String, bool) {
        self.clone().into()
    }

    pub fn as_partial(&self) -> PartialTask {
        self.clone().into()
    }
}

impl From<Task> for (String, String, bool) {
    fn from(task: Task) -> Self {
        (
            format!(
                "【{}】{}{}",
                task.category,
                match &task.subject {
                    Subject::Set(s) => format!("{} ", s),
                    Subject::Unset => "".to_string(),
                },
                task.details
            ),
            format!(
                "<t:{}:F>(<t:{}:R>)",
                task.datetime.timestamp(),
                task.datetime.timestamp()
            ),
            false,
        )
    }
}

impl From<Task> for PartialTask {
    fn from(task: Task) -> Self {
        Self {
            category: Some(task.category),
            subject: Some(task.subject.clone()),
            details: Some(task.details.clone()),
            date: Some(task.datetime.date_naive()),
            time: Some(task.datetime.time()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PartialTask {
    pub category: Option<Category>,
    pub subject: Option<Subject>,
    pub details: Option<String>,
    pub date: Option<NaiveDate>,
    pub time: Option<NaiveTime>,
}

impl PartialTask {
    pub fn unpartial(&self) -> Result<Task, Error> {
        let category = self.category.context("Category not selected")?;
        let subject = self.subject.clone().context("Subject not selected")?;
        let details = self.details.clone().context("Details not selected")?;
        let date = self.date.context("Date not selected")?;
        let time = self.time.context("Time not selected")?;
        let datetime = Local
            .from_local_datetime(&date.and_time(time))
            .single()
            .context("Invalid date and time")?;
        Ok(Task {
            category,
            subject,
            details,
            datetime,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Data {
    pub tasks: Mutex<BTreeSet<Task>>,
    pub subjects: Mutex<BTreeSet<String>>,
    pub suggest_times: Mutex<BTreeMap<NaiveTime, String>>,
    pub panel_message: Mutex<Option<(MessageId, ChannelId)>>,
    pub ping_channel: Mutex<Option<ChannelId>>,
    pub ping_role: Mutex<Option<RoleId>>,
    pub stop_ping_until: Mutex<DateTime<Local>>,
    pub log_channel: Mutex<Option<ChannelId>>,
    #[serde(skip)]
    pub panel_listener: Mutex<Option<tokio::task::JoinHandle<Result<(), Error>>>>,
}

pub const FILE_PATH: &str = "data.json";

pub fn save(data: &Data) -> Result<(), Error> {
    let data = serde_json::to_string(data)?;
    fs::write(FILE_PATH, data)?;
    Ok(())
}

pub fn load() -> Result<Data, Error> {
    let data = fs::read_to_string(FILE_PATH)?;
    let data = serde_json::from_str(&data).expect("Failed to parse data.json");
    Ok(data)
}
