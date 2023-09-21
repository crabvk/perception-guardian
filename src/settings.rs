use crate::l10n::Language;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::sync::{Mutex, OnceLock};
use std::{
    collections::HashMap, convert::TryFrom, error, fmt, num::ParseIntError, ops::Deref,
    str::FromStr,
};
use teloxide::types::ChatId;
use tokio::sync::OnceCell;

type Settings = HashMap<ChatId, HashMap<SettingKind, Setting>>;

static SETTINGS: OnceLock<Mutex<Settings>> = OnceLock::new();
static SQLITE_POOL: OnceCell<Pool<Sqlite>> = OnceCell::const_new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingKind {
    Language,
    BanChannels,
    WelcomeMessage,
}

impl From<i64> for SettingKind {
    fn from(value: i64) -> Self {
        use SettingKind::*;
        match value {
            0 => Language,
            1 => BanChannels,
            2 => WelcomeMessage,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BanChannels {
    All,
    AllExceptLinked(i64),
}

impl fmt::Display for BanChannels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::All => write!(f, "0"),
            Self::AllExceptLinked(chat_id) => write!(f, "{chat_id}"),
        }
    }
}

impl FromStr for BanChannels {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Self::All),
            value => Ok(Self::AllExceptLinked(value.parse()?)),
        }
    }
}

// Represents all possible errors that can occur when parsing a setting.
#[derive(Debug)]
pub enum SettingError {
    UnknownLanguage(String),
    ParseIntError(String),
}

impl fmt::Display for SettingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SettingError as SE;
        match self {
            SE::UnknownLanguage(lang) => write!(f, "unknown language string \"{lang}\""),
            SE::ParseIntError(value) => write!(f, "failed conversion \"{value}\" to i64"),
        }
    }
}

impl error::Error for SettingError {}

#[derive(Debug, Clone)]
pub enum Setting {
    Language(Language),
    BanChannels(BanChannels),
    WelcomeMessage(String),
}

impl TryFrom<(SettingKind, String)> for Setting {
    type Error = SettingError;

    fn try_from((kind, value): (SettingKind, String)) -> Result<Self, Self::Error> {
        use SettingKind as Kind;
        match kind {
            Kind::Language => {
                let lang: Language = value.parse()?;
                Ok(Self::Language(lang))
            }
            Kind::BanChannels => {
                let value = value
                    .parse()
                    .map_err(|_| SettingError::ParseIntError(value))?;
                Ok(Self::BanChannels(value))
            }
            Kind::WelcomeMessage => Ok(Self::WelcomeMessage(value)),
        }
    }
}

impl fmt::Display for Setting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Language(lang) => write!(f, "{lang}"),
            Self::BanChannels(val) => write!(f, "{val}"),
            Self::WelcomeMessage(msg) => write!(f, "{msg}"),
        }
    }
}

impl Setting {
    pub fn kind(&self) -> SettingKind {
        match self {
            Self::Language(_) => SettingKind::Language,
            Self::BanChannels(_) => SettingKind::BanChannels,
            Self::WelcomeMessage(_) => SettingKind::WelcomeMessage,
        }
    }
}

pub async fn preload() -> Result<(), sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect("data.db")
        .await?;

    let rows: Vec<(i64, i64, String)> = sqlx::query_as("SELECT * FROM settings")
        .fetch_all(&pool)
        .await?;

    let mut settings = HashMap::new();
    for (chat_id, kind, value) in rows {
        let kind = SettingKind::from(kind);

        // Presuming that data in database is always valid, thus `unwrap()` can't fail.
        let setting = Setting::try_from((kind, value)).unwrap();
        settings
            .entry(ChatId(chat_id))
            .or_insert(HashMap::new())
            .entry(kind)
            .or_insert(setting);
    }

    SQLITE_POOL.set(pool).unwrap();
    SETTINGS.set(Mutex::new(settings)).unwrap();

    Ok(())
}

pub async fn set(chat_id: ChatId, setting: Setting) -> Result<(), sqlx::Error> {
    let pool = SQLITE_POOL.get().unwrap();
    sqlx::query(
        r#"
INSERT INTO settings VALUES (?1, ?2, ?3)
ON CONFLICT (chat_id, setting_kind) DO UPDATE SET value = ?3
        "#,
    )
    .bind(chat_id.0)
    .bind(setting.kind() as i64)
    .bind(setting.to_string())
    .execute(pool)
    .await?;

    let mut settings = SETTINGS.get().unwrap().lock().unwrap();
    settings
        .entry(chat_id)
        .or_insert(HashMap::new())
        .insert(setting.kind(), setting);

    Ok(())
}

pub async fn delete(chat_id: ChatId, kind: SettingKind) -> Result<(), sqlx::Error> {
    let pool = SQLITE_POOL.get().unwrap();
    sqlx::query("DELETE FROM settings WHERE chat_id = ?1 AND setting_kind = ?2")
        .bind(chat_id.0)
        .bind(kind as i64)
        .execute(pool)
        .await?;

    let mut settings = SETTINGS.get().unwrap().lock().unwrap();
    settings
        .entry(chat_id)
        .or_insert(HashMap::new())
        .remove(&kind);

    Ok(())
}

pub fn get(chat_id: ChatId, kind: SettingKind) -> Option<Setting> {
    let settings = SETTINGS.get().unwrap().lock().unwrap();
    let setting = settings.get(&chat_id)?.get(&kind)?.deref().clone();
    Some(setting)
}

pub fn lang(chat_id: ChatId) -> Language {
    if let Some(Setting::Language(lang)) = get(chat_id, SettingKind::Language) {
        lang
    } else {
        Language::default()
    }
}
