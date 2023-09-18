use crate::settings::ParseSettingError;
use crate::t;
use fluent_bundle::{
    concurrent::FluentBundle, types::FluentNumber, FluentArgs, FluentResource, FluentValue,
};
use std::{collections::HashMap, env, fmt, str::FromStr};
use tokio::{fs, sync::OnceCell};

static BUNDLES: OnceCell<HashMap<&Language, FluentBundle<FluentResource>>> = OnceCell::const_new();

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    #[default]
    En,
    Ru,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::En => write!(f, "en"),
            Self::Ru => write!(f, "ru"),
        }
    }
}

impl FromStr for Language {
    type Err = ParseSettingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en" => Ok(Self::En),
            "ru" => Ok(Self::Ru),
            lang => Err(ParseSettingError::UnknownLanguage(lang.to_owned())),
        }
    }
}

impl Language {
    pub fn values() -> &'static [Self] {
        &[Self::En, Self::Ru]
    }
}

pub async fn load_locales() -> Result<(), crate::Error> {
    let mut bundles = HashMap::new();

    for lang in Language::values() {
        let lang_id = lang.to_string().parse()?;
        let mut path = env::current_dir()?;
        path.push("locales");
        path.push(format!("{lang}.ftl"));
        let source = fs::read_to_string(path).await?;
        let resource = FluentResource::try_new(source).expect("Could not parse an FTL string.");
        let mut bundle = FluentBundle::new_concurrent(vec![lang_id]);
        bundle
            .add_resource(resource)
            .expect("Failed to add FTL resources to the bundle.");
        bundle
            .add_function("DURATION", |args, _| {
                if let Some(FluentValue::Number(FluentNumber { value, .. })) = args.get(0) {
                    return format_duration(*lang, *value as u64).into();
                }

                FluentValue::None
            })
            .expect("Failed to add a function to the bundle.");
        bundles.insert(lang, bundle);
    }

    if BUNDLES.set(bundles).is_err() {
        panic!("Couldn't set BUNDLES cell.");
    }

    Ok(())
}

pub fn translate(message: &str, lang: Language, args: Option<&FluentArgs<'_>>) -> String {
    let bundle = BUNDLES.get().unwrap().get(&lang).unwrap();
    let pattern = bundle.get_message(message).unwrap().value().unwrap();
    // TODO: Handle format errors.
    let mut errors = vec![];
    let value = bundle.format_pattern(pattern, args, &mut errors);
    value.to_string()
}

fn format_duration(lang: Language, secs: u64) -> String {
    let hours = secs / 3600;
    let minutes = secs % 3600 / 60;
    let seconds = secs % 3600 % 60;

    let mut result = vec![];
    if hours > 0 {
        result.push(t!("duration-hours", lang, value = hours));
    }
    if minutes > 0 {
        result.push(t!("duration-minutes", lang, value = minutes));
    }
    if seconds > 0 {
        result.push(t!("duration-seconds", lang, value = seconds));
    }
    result.join(" ")
}

#[macro_export]
macro_rules! t {
    // t!("message-id", lang)
    ($id:expr, $lang:expr) => {
        {
            crate::l10n::translate($id, $lang, None)
        }
    };

    // t!("message-id", lang, arg1, arg2)
    ($id:expr, $lang:expr, $($arg_key_val:tt),*) => {
        {
            let mut args = fluent_bundle::FluentArgs::new();
            $(
                let arg_key = stringify!($arg_key_val);
                args.set(arg_key, $arg_key_val);
            )*
            crate::l10n::translate($id, $lang, Some(&args))
        }
    };

    // t!("message-id", lang, arg1 = 1, arg2 = "Foo")
    ($id:expr, $lang:expr, $($arg_key:tt = $arg_val:expr),*) => {
        {
            let mut args = fluent_bundle::FluentArgs::new();
            $(
                let arg_key = stringify!($arg_key);
                args.set(arg_key, $arg_val);
            )*
            crate::l10n::translate($id, $lang, Some(&args))
        }
    };
}
