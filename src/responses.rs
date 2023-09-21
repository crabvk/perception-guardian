use crate::l10n::Language;
use std::{
    error, fmt,
    str::{FromStr, ParseBoolError},
};

#[derive(Debug)]
pub struct UnknownLanguageError;

impl fmt::Display for UnknownLanguageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown language string")
    }
}

impl error::Error for UnknownLanguageError {}

pub struct LanguageResponse(pub Language);

impl FromStr for LanguageResponse {
    type Err = UnknownLanguageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en" => Ok(Self(Language::En)),
            "ru" => Ok(Self(Language::Ru)),
            _ => Err(UnknownLanguageError),
        }
    }
}

pub struct BanChannelsResponse(pub bool);

impl FromStr for BanChannelsResponse {
    type Err = ParseBoolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(BanChannelsResponse(s.parse()?))
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

pub struct WelcomeMessageResponse(pub String);

impl FromStr for WelcomeMessageResponse {
    type Err = UserTagNotPresentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("{user_tag}") {
            Ok(Self(s.to_owned()))
        } else {
            Err(UserTagNotPresentError)
        }
    }
}
