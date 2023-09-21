mod config;
mod l10n;
mod qna;
mod qwant;
mod redis;
mod responses;
mod settings;
mod utils;

use crate::config::Config;
use crate::responses::{BanChannelsResponse, LanguageResponse, WelcomeMessageResponse};
use crate::settings::{BanChannels, Setting, SettingKind};
use std::{collections::HashMap, future::IntoFuture, sync::Arc, time::Duration};
use strfmt::strfmt;
use teloxide::{
    adaptors::DefaultParseMode,
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::{
        ChatKind, ChatPermissions, ChatPublic, InputFile, Me, MessageId, ParseMode, PublicChatKind,
        PublicChatSupergroup, Update, User, UserId,
    },
    update_listeners::UpdateListener,
    update_listeners::{polling_default, webhooks},
    utils::{command::BotCommands, html},
};

const SETTINGS_BUTTONS: [(&str, &str); 3] = [
    ("Language", "0"),
    ("Ban channels", "1"),
    ("Welcome message", "2"),
];
const LANGUAGES_BUTTONS: [(&str, &str); 2] = [("🇬🇧", "en"), ("🇷🇺", "ru")];
const BAN_CHANNELS_BUTTONS: [(&str, &str); 2] = [("Yes", "true"), ("No", "false")];

type Bot = DefaultParseMode<teloxide::prelude::Bot>;
type SettingsDialogue = Dialogue<SettingsState, InMemStorage<SettingsState>>;
type Error = Box<dyn std::error::Error + Send + Sync>;
type HandlerResult = Result<(), Error>;

#[derive(Default, Clone)]
pub enum SettingsState {
    #[default]
    Start,
    SelectSetting {
        user_id: UserId,
        message_id: MessageId,
    },
    SelectValue {
        user_id: UserId,
        message_id: MessageId,
        setting_kind: SettingKind,
    },
    InputValue {
        user_id: UserId,
        message_id: MessageId,
        setting_kind: SettingKind,
    },
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "display this text")]
    Help,
    #[command(description = "change bot settings")]
    Settings,
    #[command(description = "cancel editing a setting")]
    Cancel,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot...");
    let config = Config::new().expect("Configuration error");

    // Preload Fluent bundles.
    l10n::load_locales()
        .await
        .expect("Couldn't preload Fluent bundles");

    // Setup Redis connection manager.
    redis::setup(config.redis_url.clone())
        .await
        .expect("Couldn't create Redis connection manager");

    // Preload all settings from SQLite database.
    settings::preload()
        .await
        .expect("Couldn't preload settings");

    let token = &config.telegram.token;
    let bot = teloxide::prelude::Bot::new(token).parse_mode(ParseMode::Html);

    if let Some(ref host) = config.telegram.webhook_host {
        log::info!("Receiving updates via webhook on {}", host);
        let addr = config.telegram.webhook_addr.unwrap();
        let url = format!("https://{host}/webhook").parse().unwrap();
        let opts = webhooks::Options::new(addr, url).secret_token(token.replace(":", "_"));
        let listener = webhooks::axum(bot.clone(), opts)
            .await
            .expect("Couldn't setup webhook");
        build_dispatcher(bot, schema(), listener, config).await;
    } else {
        log::info!("Using long polling to fetch updates");
        let listener = polling_default(bot.clone()).await;
        build_dispatcher(bot, schema(), listener, config).await;
    };
}

async fn build_dispatcher<UListener>(
    bot: Bot,
    handler: UpdateHandler<Error>,
    update_listener: UListener,
    config: Config,
) where
    UListener: UpdateListener,
    UListener::Err: core::fmt::Debug,
{
    let error_handler = LoggingErrorHandler::with_custom_text("An error from the update listener");
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            InMemStorage::<SettingsState>::new(),
            Arc::new(config)
        ])
        .default_handler(|_upd| async move {
            // log::warn!("Unhandled update: {:?}", upd);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(update_listener, error_handler)
        .await;
}

fn schema() -> UpdateHandler<Error> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(help_command_handler))
        .filter_async(is_user_privileged)
        .branch(
            case![SettingsState::Start]
                .branch(case![Command::Settings].endpoint(settings_command_handler)),
        )
        .branch(case![Command::Cancel].endpoint(cancel_handler));

    let message_handler = Update::filter_message()
        .branch(Message::filter_new_chat_members().endpoint(new_chat_members_handler))
        .branch(command_handler)
        .branch(
            case![SettingsState::InputValue {
                user_id,
                message_id,
                setting_kind
            }]
            .endpoint(input_setting_value_handler),
        )
        .filter(is_channel_message)
        .endpoint(channel_message_handler);

    let callback_settings_handler = Update::filter_callback_query()
        .branch(
            case![SettingsState::SelectSetting {
                user_id,
                message_id
            }]
            .endpoint(select_setting_kind_handler),
        )
        .branch(
            case![SettingsState::SelectValue {
                user_id,
                message_id,
                setting_kind
            }]
            .endpoint(select_setting_value_handler),
        );

    dialogue::enter::<Update, InMemStorage<SettingsState>, SettingsState, _>()
        .filter(is_group_or_supergroup)
        .branch(message_handler)
        .branch(callback_settings_handler)
        .branch(Update::filter_callback_query().endpoint(captcha_response_handler))
}

fn is_channel_message(upd: Update) -> bool {
    if let Some(user) = upd.user() {
        user.id.is_channel()
    } else {
        false
    }
}

fn is_group_or_supergroup(upd: Update) -> bool {
    if let Some(chat) = upd.chat() {
        chat.is_group() || chat.is_supergroup()
    } else {
        false
    }
}

async fn is_user_privileged(bot: Bot, upd: Update) -> bool {
    if let (Some(chat), Some(user)) = (upd.chat(), upd.user()) {
        let member = bot.get_chat_member(chat.id, user.id).await;
        if let Ok(member) = member {
            return member.kind.is_privileged();
        }
    }

    false
}

async fn channel_message_handler(bot: Bot, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id;
    let ban_channels = settings::get(chat_id, SettingKind::BanChannels);

    if ban_channels.is_none() {
        return Ok(());
    }

    // `sender_chat()` is `Some(_)` because of `is_channel_message` filter in `schema()`.
    let sender_chat = msg.sender_chat().unwrap();
    let linked_chat_id = if let Setting::BanChannels(BanChannels::AllExceptLinked(linked_chat_id)) =
        ban_channels.unwrap()
    {
        linked_chat_id
    } else {
        0
    };

    if linked_chat_id != sender_chat.id.0 {
        let _ = tokio::join!(
            bot.ban_chat_sender_chat(chat_id, sender_chat.id)
                .into_future(),
            bot.delete_message(chat_id, msg.id).into_future()
        );
    }

    Ok(())
}

async fn cancel_handler(
    cfg: Arc<Config>,
    bot: Bot,
    msg: Message,
    dialogue: SettingsDialogue,
) -> HandlerResult {
    let state = dialogue.get().await?;

    if state.is_none() {
        return Ok(());
    }

    if let Some(
        SettingsState::SelectSetting {
            user_id,
            message_id,
        }
        | SettingsState::SelectValue {
            user_id,
            message_id,
            ..
        }
        | SettingsState::InputValue {
            user_id,
            message_id,
            ..
        },
    ) = state
    {
        if user_id == msg.from().unwrap().id {
            let chat_id = msg.chat.id;
            let lang = settings::lang(chat_id);
            let text = t!("settings-cancel", lang);
            let (message, _) = tokio::join!(
                bot.send_message(chat_id, text).into_future(),
                bot.delete_message(chat_id, message_id).into_future()
            );

            if let Ok(message) = message {
                let _ = utils::delete_message_later(
                    &bot,
                    chat_id,
                    message.id,
                    Duration::from_secs(cfg.guardian.message_expire),
                );
            }

            dialogue.exit().await?;
        }
    }

    Ok(())
}

async fn input_setting_value_handler(
    cfg: Arc<Config>,
    bot: Bot,
    msg: Message,
    dialogue: SettingsDialogue,
    (user_id, _, _): (UserId, MessageId, SettingKind),
) -> HandlerResult {
    if user_id != msg.from().unwrap().id {
        return Ok(());
    }

    let chat_id = msg.chat.id;
    let lang = settings::lang(chat_id);
    let text = msg.text();

    if text.is_none() {
        let text = t!("settings-text-required", lang);
        bot.send_message(chat_id, text).await?;
        return Ok(());
    }

    let response: Result<WelcomeMessageResponse, _> = text.unwrap().parse();

    if let Err(error) = response {
        let text = format!("Couldn't parse <b>WelcomeMessage</b> value: {error}");
        bot.send_message(chat_id, text).await?;
        return Ok(());
    }

    let welcome_message = response.unwrap().0;
    let text = t!(
        "settings-welcome-message-set",
        lang,
        welcome_message = &welcome_message
    );
    let setting = Setting::WelcomeMessage(welcome_message);

    // Send welcome message back to check its validity.
    match bot.send_message(chat_id, text).await {
        Ok(message) => {
            settings::set(chat_id, setting).await?;
            let _ = utils::delete_message_later(
                &bot,
                chat_id,
                message.id,
                Duration::from_secs(cfg.guardian.message_expire),
            );
            dialogue.exit().await?;
        }
        Err(error) => {
            bot.send_message(chat_id, error.to_string()).await?;
        }
    }

    Ok(())
}

async fn select_setting_value_handler(
    cfg: Arc<Config>,
    bot: Bot,
    dialogue: SettingsDialogue,
    query: CallbackQuery,
    upd: Update,
    (user_id, message_id, setting_kind): (UserId, MessageId, SettingKind),
) -> HandlerResult {
    if query.data.is_none() {
        return Ok(());
    }

    let chat_id = dialogue.chat_id();
    let lang = settings::lang(chat_id);
    let message = query.message;

    if message.is_none() || message_id != message.as_ref().unwrap().id {
        let text = t!("settings-message-outdated", lang);
        bot.answer_callback_query(query.id).text(text).await?;
        return Ok(());
    }

    if user_id != upd.user().unwrap().id {
        let text = t!("query-wrong-user", lang);
        bot.answer_callback_query(query.id).text(text).await?;
        return Ok(());
    }

    bot.answer_callback_query(query.id).await?;
    let value = query.data.unwrap();

    let send_message = match setting_kind {
        SettingKind::Language => {
            let response: Result<LanguageResponse, _> = value.parse();

            if let Err(error) = response {
                let text = format!("Couldn't parse <b>Language</b> value: {error}");
                bot.send_message(chat_id, text).await?;
                return Ok(());
            }

            let lang = response.unwrap().0;
            let setting = Setting::Language(lang);
            settings::set(chat_id, setting).await?;
            let text = t!("settings-language-set", lang, lang);
            bot.send_message(chat_id, text)
        }
        SettingKind::BanChannels => {
            let response: Result<BanChannelsResponse, _> = value.parse();

            if let Err(error) = response {
                let text = format!("Couldn't parse <b>BanChannels</b> value: {error}");
                bot.send_message(chat_id, text).await?;
                return Ok(());
            }

            if response.unwrap().0 {
                let chat = bot.get_chat(chat_id).await?;

                if let ChatKind::Public(ChatPublic {
                    kind:
                        PublicChatKind::Supergroup(PublicChatSupergroup {
                            linked_chat_id: Some(linked_chat_id),
                            ..
                        }),
                    ..
                }) = chat.kind
                {
                    let setting =
                        Setting::BanChannels(BanChannels::AllExceptLinked(linked_chat_id));
                    settings::set(chat_id, setting).await?;
                    let text = t!("settings-ban-channels-linked-set", lang, linked_chat_id);
                    bot.send_message(chat_id, text)
                } else {
                    let setting = Setting::BanChannels(BanChannels::All);
                    settings::set(chat_id, setting).await?;
                    let text = t!("settings-ban-channels-set", lang);
                    bot.send_message(chat_id, text)
                }
            } else {
                let ban_channels = settings::get(chat_id, SettingKind::BanChannels);
                if ban_channels.is_some() {
                    settings::delete(chat_id, SettingKind::BanChannels).await?;
                }
                let text = t!("settings-ban-channels-none-set", lang);
                bot.send_message(chat_id, text)
            }
        }
        _ => unreachable!(),
    };

    let (message, _) = tokio::join!(
        send_message.into_future(),
        bot.delete_message(chat_id, message_id).into_future()
    );
    if let Ok(message) = message {
        let _ = utils::delete_message_later(
            &bot,
            chat_id,
            message.id,
            Duration::from_secs(cfg.guardian.message_expire),
        );
    }
    dialogue.exit().await?;
    Ok(())
}

async fn select_setting_kind_handler(
    bot: Bot,
    dialogue: SettingsDialogue,
    query: CallbackQuery,
    upd: Update,
    (user_id, message_id): (UserId, MessageId),
) -> HandlerResult {
    if query.data.is_none() {
        return Ok(());
    }

    let chat_id = dialogue.chat_id();
    let lang = settings::lang(chat_id);
    let message = query.message;

    if message.is_none() || message_id != message.as_ref().unwrap().id {
        let text = t!("settings-message-outdated", lang);
        bot.answer_callback_query(query.id).text(text).await?;
        return Ok(());
    }

    if user_id != upd.user().unwrap().id {
        let text = t!("query-wrong-user", lang);
        bot.answer_callback_query(query.id).text(text).await?;
        return Ok(());
    }

    bot.answer_callback_query(query.id).await?;
    bot.delete_message(chat_id, message.unwrap().id).await?;
    let setting_kind = query.data.unwrap().parse::<i64>();

    if let Err(error) = setting_kind {
        let text = format!("Couldn't parse callback query data: {error}");
        bot.send_message(chat_id, text).await?;
        return Ok(());
    }

    let setting_kind = SettingKind::from(setting_kind.unwrap());
    let user_id = query.from.id;
    match setting_kind {
        SettingKind::Language | SettingKind::BanChannels => {
            let setting = settings::get(chat_id, setting_kind);
            let (keyboard, text) = if setting_kind == SettingKind::Language {
                let text = if let Some(Setting::Language(lang)) = setting {
                    t!("settings-select-language", lang)
                } else {
                    t!("settings-select-language-default", lang)
                };
                (utils::keyboard(&LANGUAGES_BUTTONS, 2), text)
            } else if setting_kind == SettingKind::BanChannels {
                let text = match setting {
                    Some(Setting::BanChannels(BanChannels::All)) => {
                        t!("settings-select-ban-channels-all", lang)
                    }
                    Some(Setting::BanChannels(BanChannels::AllExceptLinked(linked_chat_id))) => {
                        t!("settings-select-ban-channels-linked", lang, linked_chat_id)
                    }
                    None => t!("settings-select-ban-channels-none", lang),
                    _ => unreachable!(),
                };
                (utils::keyboard(&BAN_CHANNELS_BUTTONS, 2), text)
            } else {
                unreachable!()
            };
            let message = bot
                .send_message(chat_id, text)
                .reply_markup(keyboard)
                .await?;
            dialogue
                .update(SettingsState::SelectValue {
                    user_id,
                    setting_kind,
                    message_id: message.id,
                })
                .await?;
        }
        SettingKind::WelcomeMessage => {
            let text = t!("settings-input-welcome-message", lang);
            let message = bot
                .send_message(chat_id, text)
                .disable_web_page_preview(true)
                .await?;
            dialogue
                .update(SettingsState::InputValue {
                    user_id,
                    setting_kind,
                    message_id: message.id,
                })
                .await?;
        }
    }

    Ok(())
}

async fn settings_command_handler(
    bot: Bot,
    msg: Message,
    dialogue: SettingsDialogue,
) -> HandlerResult {
    let chat_id = msg.chat.id;
    let lang = settings::lang(chat_id);
    let text = t!("settings-select-kind", lang);
    let keyboard = utils::keyboard(&SETTINGS_BUTTONS, 2);
    let message = bot
        .send_message(chat_id, text)
        .reply_markup(keyboard)
        .reply_to_message_id(msg.id)
        .await?;
    dialogue
        .update(SettingsState::SelectSetting {
            user_id: msg.from().unwrap().id,
            message_id: message.id,
        })
        .await?;

    Ok(())
}

async fn help_command_handler(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .reply_to_message_id(msg.id)
        .await?;
    Ok(())
}

async fn new_chat_members_handler(
    cfg: Arc<Config>,
    bot: Bot,
    msg: Message,
    chat_members: Vec<User>,
    me: Me,
) -> HandlerResult {
    let chat_id = msg.chat.id;
    let lang = settings::lang(chat_id);
    let users = chat_members.iter().filter(|m| !m.is_bot);

    // Restrict new users as soon as possible.
    let users_futures = users.clone().map(|user| {
        bot.restrict_chat_member(chat_id, user.id, ChatPermissions::empty())
            .into_future()
    });
    let restrictions = futures::future::join_all(users_futures).await;

    // Show error from the first failed restriction (if any).
    let failed_restrictions: Vec<_> = restrictions.iter().filter(|r| r.is_err()).collect();
    if failed_restrictions.len() > 0 {
        if let Err(error) = failed_restrictions[0] {
            let message = format!("Couldn't restrict user: {error}");
            log::error!("{message}");
            bot.send_message(chat_id, message).await?;
        }
    }

    let bots = chat_members.iter().filter(|m| m.is_bot);
    for b in bots {
        if b.id == me.id {
            let text = t!("make-me-admin", lang);
            bot.send_message(chat_id, text).await?;
        }
        log::info!(
            "New member {} is a bot, skipping captcha",
            b.username.as_ref().unwrap()
        );
    }

    let restricted_users = users.enumerate().filter(|(i, _)| restrictions[*i].is_ok());
    for (_, user) in restricted_users {
        // Skip showing CAPTCHA for ignored users.
        if redis::is_ignored(chat_id, user.id).await {
            log::info!("Ignoring user {} in chat {}", user.id, chat_id);
            continue;
        }

        let comb = qna::Combination::pick(6);
        log::info!("{comb}");

        // TODO: Handle request errors from Qwant.com
        let user_tag = html::user_mention_or_link(user);
        let url = qwant::get_image_url(comb.query_phrase).await?;
        log::info!("Image URL: {url}");
        let keyboard = utils::emojis_keyboard(&comb.emojis, 2);
        let caption = t!(
            "captcha-caption",
            lang,
            user_tag = &user_tag,
            expire = cfg.guardian.captcha_expire
        );
        let message = bot
            .send_photo(chat_id, InputFile::url(url))
            .caption(caption)
            .reply_markup(keyboard)
            .await?;
        let text = t!(
            "captcha-time-over",
            lang,
            user_tag = user_tag,
            duration = cfg.guardian.ignore_expire
        );
        let _ = utils::delete_captcha_later(
            &bot,
            chat_id,
            message.id,
            text,
            Duration::from_secs(cfg.guardian.captcha_expire),
            Duration::from_secs(cfg.guardian.message_expire),
        );
        redis::set_answer(
            chat_id,
            user.id,
            comb.answer,
            cfg.guardian.captcha_expire,
            cfg.guardian.ignore_expire,
        )
        .await?;
    }

    Ok(())
}

async fn captcha_response_handler(
    cfg: Arc<Config>,
    bot: Bot,
    query: CallbackQuery,
) -> HandlerResult {
    if query.data.is_none() || query.message.is_none() {
        return Ok(());
    }

    let answer = query.data.unwrap();
    let message = query.message.unwrap();
    let chat_id = message.chat.id;
    let user_id = query.from.id;
    let lang = settings::lang(chat_id);
    let correct_answer = redis::get_answer(chat_id, user_id).await?;

    if correct_answer.is_none() {
        let text = t!("query-wrong-user", lang);
        bot.answer_callback_query(query.id).text(text).await?;
        return Ok(());
    }

    let correct_answer = correct_answer.unwrap();
    let user_tag = html::user_mention_or_link(&query.from);

    if *answer == correct_answer {
        let text = t!("query-correct", lang);
        let (restriction, _, _) = tokio::join!(
            bot.restrict_chat_member(chat_id, user_id, ChatPermissions::all())
                .into_future(),
            bot.answer_callback_query(query.id).text(text).into_future(),
            bot.delete_message(chat_id, message.id).into_future()
        );

        // Show error if restriction didn't work.
        if let Err(error) = restriction {
            let message = format!("Couldn't restrict user: {error}");
            log::error!("{message}");
            bot.send_message(chat_id, message).await?;
            return Ok(());
        }

        let text = if let Some(Setting::WelcomeMessage(text)) =
            settings::get(chat_id, SettingKind::WelcomeMessage)
        {
            let mut vars = HashMap::new();
            vars.insert("user_tag".to_string(), user_tag);
            strfmt(&text, &vars).unwrap()
        } else {
            t!("welcome", lang, user_tag)
        };
        let message = bot.send_message(chat_id, text).await?;
        let _ = utils::delete_message_later(
            &bot,
            chat_id,
            message.id,
            Duration::from_secs(cfg.guardian.message_expire),
        );
    } else {
        let text = t!("query-wrong", lang);
        let _ = tokio::join!(
            bot.answer_callback_query(query.id).text(text).into_future(),
            bot.delete_message(chat_id, message.id).into_future()
        );
        let text = t!(
            "captcha-incorrect-answer",
            lang,
            user_tag = user_tag,
            duration = cfg.guardian.ignore_expire
        );
        let message = bot.send_message(chat_id, text).await?;
        let _ = utils::delete_message_later(
            &bot,
            chat_id,
            message.id,
            Duration::from_secs(cfg.guardian.message_expire),
        );
    }

    Ok(())
}
