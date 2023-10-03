use crate::l10n::Language;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::num::NonZeroU64;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use std::{error, fmt, str};
use teloxide::types::ChatId;
use tokio::sync::OnceCell;

static SETTINGS: OnceLock<Mutex<HashMap<ChatId, Settings>>> = OnceLock::new();
static GREETINGS: OnceLock<Mutex<HashMap<ChatId, String>>> = OnceLock::new();
static SQLITE_POOL: OnceCell<Pool<Sqlite>> = OnceCell::const_new();

#[derive(Debug, Clone)]
pub struct Settings {
    pub language: Language,
    pub ban_channels: Option<BanChannels>,
    pub captcha_expire: NonZeroU64,
    pub message_expire: NonZeroU64,
    pub ignore_expire: NonZeroU64,
    pub delete_entry_messages: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::default(),
            ban_channels: None,
            captcha_expire: NonZeroU64::new(60).unwrap(),
            message_expire: NonZeroU64::new(10).unwrap(),
            ignore_expire: NonZeroU64::new(300).unwrap(),
            delete_entry_messages: false,
        }
    }
}

impl Settings {
    pub fn captcha_expire(&self) -> Duration {
        Duration::from_secs(self.captcha_expire.get())
    }

    pub fn message_expire(&self) -> Duration {
        Duration::from_secs(self.message_expire.get())
    }
}

#[derive(Debug)]
pub enum RawSettingError {
    InvalidFormat(String),
    UnknownSetting(String),
    InvalidValue(anyhow::Error),
}

impl fmt::Display for RawSettingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RawSettingError as Error;
        match self {
            Error::InvalidFormat(val) => write!(f, "string \"{val}\" has invalid format"),
            Error::UnknownSetting(key) => write!(f, "unknown setting \"{key}\""),
            Error::InvalidValue(error) => write!(f, "{error}"),
        }
    }
}

impl error::Error for RawSettingError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RawSetting {
    Language(Language),
    BanChannels(bool),
    CaptchaExpire(NonZeroU64),
    MessageExpire(NonZeroU64),
    IgnoreExpire(NonZeroU64),
    DeleteEntryMessages(bool),
}

impl RawSetting {
    pub fn from_str(s: &str) -> Result<HashSet<RawSetting>, RawSettingError> {
        let mut settings = HashSet::new();

        for line in s.lines() {
            let line = line.trim();
            if line == "" {
                continue;
            }
            let key_value = line.split_once(":");
            if key_value.is_none() {
                return Err(RawSettingError::InvalidFormat(line.into()));
            }
            let (key, value) = key_value
                .map(|(key, value)| (key.trim(), value.trim()))
                .unwrap();

            match key {
                "language" => {
                    let value = value
                        .parse::<Language>()
                        .map_err(|error| RawSettingError::InvalidValue(error.into()))?;
                    settings.insert(RawSetting::Language(value));
                }
                "ban_channels" => {
                    let value = value
                        .parse::<bool>()
                        .map_err(|error| RawSettingError::InvalidValue(error.into()))?;
                    settings.insert(RawSetting::BanChannels(value));
                }
                "captcha_expire" => {
                    let value = value
                        .parse::<NonZeroU64>()
                        .map_err(|error| RawSettingError::InvalidValue(error.into()))?;
                    settings.insert(RawSetting::CaptchaExpire(value));
                }
                "message_expire" => {
                    let value = value
                        .parse::<NonZeroU64>()
                        .map_err(|error| RawSettingError::InvalidValue(error.into()))?;
                    settings.insert(RawSetting::MessageExpire(value));
                }
                "ignore_expire" => {
                    let value = value
                        .parse::<NonZeroU64>()
                        .map_err(|error| RawSettingError::InvalidValue(error.into()))?;
                    settings.insert(RawSetting::IgnoreExpire(value));
                }
                "delete_entry_messages" => {
                    let value = value
                        .parse::<bool>()
                        .map_err(|error| RawSettingError::InvalidValue(error.into()))?;
                    settings.insert(RawSetting::DeleteEntryMessages(value));
                }
                _ => return Err(RawSettingError::UnknownSetting(key.into())),
            }
        }

        Ok(settings)
    }

    pub fn to_string(settings: &Settings) -> String {
        let mut lines = Vec::with_capacity(6);
        lines.push(format!("language: <code>{}</code>", settings.language));
        lines.push(format!(
            "ban_channels: <code>{}</code>",
            settings.ban_channels.is_some()
        ));
        lines.push(format!(
            "captcha_expire: <code>{}</code>",
            settings.captcha_expire
        ));
        lines.push(format!(
            "message_expire: <code>{}</code>",
            settings.message_expire
        ));
        lines.push(format!(
            "ignore_expire: <code>{}</code>",
            settings.ignore_expire
        ));
        lines.push(format!(
            "delete_entry_messages: <code>{}</code>",
            settings.delete_entry_messages
        ));
        lines.join("\n")
    }
}

#[derive(Debug, Clone)]
pub enum BanChannels {
    All,
    AllExceptLinked(i64),
}

impl From<i64> for BanChannels {
    fn from(value: i64) -> Self {
        if value == 0 {
            Self::All
        } else {
            Self::AllExceptLinked(value)
        }
    }
}

impl From<&BanChannels> for i64 {
    fn from(value: &BanChannels) -> Self {
        match value {
            BanChannels::All => 0,
            BanChannels::AllExceptLinked(val) => *val,
        }
    }
}

#[derive(Debug)]
pub struct UserTagNotPresentError;

impl fmt::Display for UserTagNotPresentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "string must containt \"{{user_tag}}\" substring")
    }
}

impl error::Error for UserTagNotPresentError {}

pub struct RawGreeting(pub String);

impl str::FromStr for RawGreeting {
    type Err = UserTagNotPresentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("{user_tag}") {
            Ok(Self(s.to_owned()))
        } else {
            Err(UserTagNotPresentError)
        }
    }
}

pub async fn preload() -> Result<(), sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect("data.db")
        .await?;

    let rows = sqlx::query("SELECT * FROM settings")
        .fetch_all(&pool)
        .await?;

    let mut settings = HashMap::new();
    for row in rows {
        let chat_id: i64 = row.get("chat_id");
        let language: Language = row
            .try_get::<String, _>("language")
            .map(|val| val.parse().unwrap())
            .unwrap();
        let ban_channels: Option<BanChannels> = row
            .get::<Option<i64>, _>("ban_channels")
            .map(|val| val.into());
        let captcha_expire = row
            .try_get::<i64, _>("captcha_expire")
            .map(|val| NonZeroU64::new(val as u64).unwrap())
            .unwrap();
        let message_expire = row
            .try_get::<i64, _>("message_expire")
            .map(|val| NonZeroU64::new(val as u64).unwrap())
            .unwrap();
        let ignore_expire = row
            .try_get::<i64, _>("ignore_expire")
            .map(|val| NonZeroU64::new(val as u64).unwrap())
            .unwrap();
        let delete_entry_messages: bool = row.get("delete_entry_messages");

        settings.insert(
            ChatId(chat_id),
            Settings {
                language,
                ban_channels,
                captcha_expire,
                message_expire,
                ignore_expire,
                delete_entry_messages,
            },
        );
    }

    let rows: Vec<(i64, String)> = sqlx::query_as("SELECT * FROM greetings")
        .fetch_all(&pool)
        .await?;

    let mut greetings = HashMap::new();
    for row in rows {
        greetings.insert(ChatId(row.0), row.1);
    }

    SQLITE_POOL.set(pool).unwrap();
    SETTINGS.set(Mutex::new(settings)).unwrap();
    GREETINGS.set(Mutex::new(greetings)).unwrap();

    Ok(())
}

pub fn get(chat_id: ChatId) -> Settings {
    let settings = SETTINGS.get().unwrap().lock().unwrap();
    if let Some(settings) = settings.get(&chat_id) {
        settings.clone()
    } else {
        Settings::default()
    }
}

pub fn get_ban_channels(chat_id: ChatId) -> Option<BanChannels> {
    let settings = SETTINGS.get().unwrap().lock().unwrap();
    if let Some(settings) = settings.get(&chat_id) {
        settings.ban_channels.clone()
    } else {
        None
    }
}

pub fn get_greeting(chat_id: ChatId) -> Option<String> {
    let greeting = GREETINGS.get().unwrap().lock().unwrap();
    greeting.get(&chat_id).map(|val| val.clone())
}

pub async fn set(chat_id: ChatId, settings: Settings) -> Result<(), sqlx::Error> {
    let ban_channels: Option<i64> = settings.ban_channels.as_ref().map(|val| val.into());

    let pool = SQLITE_POOL.get().unwrap();
    sqlx::query(
        r#"
INSERT INTO settings VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
ON CONFLICT (chat_id) DO UPDATE SET
    language = ?2,
    ban_channels = ?3,
    captcha_expire = ?4,
    message_expire = ?5,
    ignore_expire = ?6,
    delete_entry_messages = ?7
        "#,
    )
    .bind(chat_id.0)
    .bind(settings.language.to_string())
    .bind(ban_channels)
    .bind(settings.captcha_expire.get() as i64)
    .bind(settings.message_expire.get() as i64)
    .bind(settings.ignore_expire.get() as i64)
    .bind(settings.delete_entry_messages)
    .execute(pool)
    .await?;

    let mut hm = SETTINGS.get().unwrap().lock().unwrap();
    hm.insert(chat_id, settings);

    Ok(())
}

pub async fn set_greeting(chat_id: ChatId, greeting: String) -> Result<(), sqlx::Error> {
    let pool = SQLITE_POOL.get().unwrap();
    sqlx::query(
        r#"
INSERT INTO greetings VALUES (?1, ?2)
ON CONFLICT (chat_id) DO UPDATE SET greeting = ?2
        "#,
    )
    .bind(chat_id.0)
    .bind(&greeting)
    .execute(pool)
    .await?;

    let mut hm = GREETINGS.get().unwrap().lock().unwrap();
    hm.insert(chat_id, greeting);

    Ok(())
}

pub fn lang(chat_id: ChatId) -> Language {
    let settings = SETTINGS.get().unwrap().lock().unwrap();

    if let Some(settings) = settings.get(&chat_id) {
        settings.language
    } else {
        Language::default()
    }
}
