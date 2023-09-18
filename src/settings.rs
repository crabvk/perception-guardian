use crate::l10n::Language;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::sync::{Mutex, OnceLock};
use std::{collections::HashMap, error, fmt, ops::Deref};
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
            _ => WelcomeMessage,
        }
    }
}

// Represents all possible errors that can occur when parsing a setting.
#[derive(Debug)]
pub enum ParseSettingError {
    UnknownLanguage(String),
    BoolError(String),
    UserTagNotPresent,
    #[allow(dead_code)]
    DisallowedHtmlEntity(String),
}

impl fmt::Display for ParseSettingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseSettingError as SE;
        match self {
            SE::UnknownLanguage(lang) => write!(f, "unknown language string \"{lang}\""),
            SE::BoolError(str) => write!(f, "`true` or `false` expected, got \"{str}\""),
            SE::UserTagNotPresent => write!(f, "string must containt \"{{user_tag}}\" substring"),
            SE::DisallowedHtmlEntity(_error) => todo!(),
        }
    }
}

impl error::Error for ParseSettingError {}

#[derive(Debug, Clone)]
pub enum Setting {
    Language(Language),
    BanChannels(bool),
    WelcomeMessage(String),
}

impl Setting {
    pub const SETTINGS_VARIANTS: [(&str, &str); 3] = [
        ("Language", "0"),
        ("Ban channels", "1"),
        ("Welcome message", "2"),
    ];
    pub const LANGUAGES_VARIANTS: [(&str, &str); 2] = [("🇬🇧", "en"), ("🇷🇺", "ru")];
    pub const BAN_CHANNELS_VARIANTS: [(&str, &str); 2] = [("Yes", "true"), ("No", "false")];

    pub fn value(&self) -> String {
        match self {
            Self::Language(lang) => lang.to_string(),
            Self::BanChannels(val) => val.to_string(),
            Self::WelcomeMessage(msg) => msg.to_string(),
        }
    }

    pub fn parse(kind: SettingKind, value: &str) -> Result<Self, ParseSettingError> {
        use SettingKind as SK;
        match kind {
            SK::Language => {
                let lang: Language = value.parse()?;
                Ok(Self::Language(lang))
            }
            SK::BanChannels => {
                if let Ok(value) = value.parse() {
                    Ok(Self::BanChannels(value))
                } else {
                    Err(ParseSettingError::BoolError(value.to_owned()))
                }
            }
            SK::WelcomeMessage => {
                if !value.contains("{user_tag}") {
                    return Err(ParseSettingError::UserTagNotPresent);
                }
                Ok(Self::WelcomeMessage(value.to_owned()))
            }
        }
    }

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
        let setting = Setting::parse(kind, &value).unwrap();
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
    .bind(setting.value())
    .execute(pool)
    .await?;

    let mut settings = SETTINGS.get().unwrap().lock().unwrap();
    settings
        .entry(chat_id)
        .or_insert(HashMap::new())
        .insert(setting.kind(), setting);

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
