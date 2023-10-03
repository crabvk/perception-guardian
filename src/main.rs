mod config;
mod l10n;
mod qna;
mod qwant;
mod redis;
mod settings;
mod utils;

use crate::config::Config;
use crate::settings::{BanChannels, RawGreeting, RawSetting};
use std::{collections::HashMap, future::IntoFuture};
use strfmt::strfmt;
use teloxide::{
    adaptors::DefaultParseMode,
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::{
        ChatKind, ChatPermissions, ChatPublic, InputFile, Me, ParseMode, PublicChatKind,
        PublicChatSupergroup, Update, User, UserId,
    },
    update_listeners::UpdateListener,
    update_listeners::{polling_default, webhooks},
    utils::{command::BotCommands, html},
};

type Bot = DefaultParseMode<teloxide::prelude::Bot>;
type SettingsDialogue = Dialogue<SettingsState, InMemStorage<SettingsState>>;
type Error = Box<dyn std::error::Error + Send + Sync>;
type HandlerResult = Result<(), Error>;

#[derive(Default, Clone)]
pub enum SettingsState {
    #[default]
    Start,
    Settings {
        user_id: UserId,
    },
    Greeting {
        user_id: UserId,
    },
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "camelCase")]
enum Command {
    #[command(description = "display this text")]
    Help,
    #[command(description = "change bot settings")]
    Settings,
    #[command(description = "change greeting of newcomers")]
    Greeting,
    #[command(description = "cancel changing settings or greeting")]
    Cancel,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    log::info!("Starting bot...");
    let config = Config::new()?;

    // Preload Fluent bundles.
    l10n::load_locales()
        .await
        .expect("Couldn't preload Fluent bundles");

    // Setup Redis connection manager.
    redis::setup(config.redis_url)
        .await
        .expect("Couldn't create Redis connection manager");

    // Preload all settings from SQLite database.
    settings::preload()
        .await
        .expect("Couldn't preload settings");

    let token = &config.token;
    let bot = teloxide::prelude::Bot::new(token).parse_mode(ParseMode::Html);

    if let Some(ref host) = config.webhook_host {
        log::info!("Receiving updates via webhook on {}", host);
        let addr = config.webhook_addr.unwrap();
        let url = format!("https://{host}/webhook").parse().unwrap();
        let opts = webhooks::Options::new(addr, url).secret_token(token.replace(":", "_"));
        let listener = webhooks::axum(bot.clone(), opts)
            .await
            .expect("Couldn't setup webhook");
        build_dispatcher(bot, schema(), listener).await;
    } else {
        log::info!("Using long polling to fetch updates");
        let listener = polling_default(bot.clone()).await;
        build_dispatcher(bot, schema(), listener).await;
    };

    Ok(())
}

async fn build_dispatcher<UListener>(
    bot: Bot,
    handler: UpdateHandler<Error>,
    update_listener: UListener,
) where
    UListener: UpdateListener,
    UListener::Err: core::fmt::Debug,
{
    let error_handler = LoggingErrorHandler::with_custom_text("An error from the update listener");
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![InMemStorage::<SettingsState>::new()])
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
        .branch(
            case![SettingsState::Start]
                .branch(case![Command::Greeting].endpoint(greeting_command_handler)),
        )
        .branch(case![Command::Cancel].endpoint(cancel_handler));

    let message_handler = Update::filter_message()
        .branch(Message::filter_new_chat_members().endpoint(new_chat_members_handler))
        .branch(Message::filter_left_chat_member().endpoint(left_chat_member_handler))
        .branch(command_handler)
        .branch(case![SettingsState::Settings { user_id }].endpoint(input_settings_handler))
        .branch(case![SettingsState::Greeting { user_id }].endpoint(input_greeting_handler))
        .filter(is_channel_message)
        .endpoint(channel_message_handler);

    dialogue::enter::<Update, InMemStorage<SettingsState>, SettingsState, _>()
        .filter(is_group_or_supergroup)
        .branch(message_handler)
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

async fn left_chat_member_handler(bot: Bot, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id;
    let settings = settings::get(chat_id);
    if settings.delete_entry_messages {
        bot.delete_message(chat_id, msg.id).await?;
    }

    Ok(())
}

async fn channel_message_handler(bot: Bot, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id;
    let ban_channels = settings::get_ban_channels(chat_id);

    if ban_channels.is_none() {
        return Ok(());
    }

    // `sender_chat()` is `Some(_)` because of `is_channel_message` filter in `schema()`.
    let sender_chat = msg.sender_chat().unwrap();
    let linked_chat_id = if let BanChannels::AllExceptLinked(linked_chat_id) = ban_channels.unwrap()
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

async fn input_settings_handler(
    bot: Bot,
    msg: Message,
    dialogue: SettingsDialogue,
    user_id: UserId,
) -> HandlerResult {
    if user_id != msg.from().unwrap().id {
        return Ok(());
    }

    let chat_id = msg.chat.id;
    let text = msg.text();
    let mut settings = settings::get(chat_id);

    if text.is_none() {
        let text = t!("settings-text-required", settings.language);
        bot.send_message(chat_id, text).await?;
        return Ok(());
    }

    let raw_settings = RawSetting::from_str(text.unwrap());

    if let Err(error) = raw_settings {
        bot.send_message(
            chat_id,
            format!("Parsing error: {error}.\nTry again or /cancel"),
        )
        .await?;
        return Ok(());
    }

    for raw_setting in raw_settings.unwrap() {
        match raw_setting {
            RawSetting::Language(lang) => settings.language = lang,
            RawSetting::BanChannels(val) => {
                let val = if val {
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
                        Some(BanChannels::AllExceptLinked(linked_chat_id))
                    } else {
                        Some(BanChannels::All)
                    }
                } else {
                    None
                };
                settings.ban_channels = val;
            }
            RawSetting::CaptchaExpire(val) => settings.captcha_expire = val,
            RawSetting::MessageExpire(val) => settings.message_expire = val,
            RawSetting::IgnoreExpire(val) => settings.ignore_expire = val,
            RawSetting::DeleteEntryMessages(val) => settings.delete_entry_messages = val,
        }
    }

    let lang = settings.language;
    let message_expire = settings.message_expire();
    settings::set(chat_id, settings).await?;
    let text = t!("settings-changed", lang);
    let message = bot.send_message(chat_id, text).await?;
    let _ = utils::delete_message_later(&bot, chat_id, message.id, message_expire);
    dialogue.exit().await?;

    Ok(())
}

async fn greeting_command_handler(
    bot: Bot,
    msg: Message,
    dialogue: SettingsDialogue,
) -> HandlerResult {
    let chat_id = msg.chat.id;
    let lang = settings::lang(chat_id);
    let greeting =
        settings::get_greeting(chat_id).unwrap_or(t!("greeting", lang, user_tag = "{user_tag}"));
    let text = t!("settings-input-greeting", lang, greeting);

    bot.send_message(chat_id, text)
        .reply_to_message_id(msg.id)
        .disable_web_page_preview(true)
        .await?;
    dialogue
        .update(SettingsState::Greeting {
            user_id: msg.from().unwrap().id,
        })
        .await?;

    Ok(())
}

async fn settings_command_handler(
    bot: Bot,
    msg: Message,
    dialogue: SettingsDialogue,
) -> HandlerResult {
    let chat_id = msg.chat.id;
    let settings = settings::get(chat_id);
    let text = RawSetting::to_string(&settings);

    bot.send_message(chat_id, text)
        .reply_to_message_id(msg.id)
        .await?;

    let text = t!("settings-input", settings.language);
    bot.send_message(chat_id, text)
        .disable_web_page_preview(true)
        .await?;
    dialogue
        .update(SettingsState::Settings {
            user_id: msg.from().unwrap().id,
        })
        .await?;

    Ok(())
}

async fn cancel_handler(bot: Bot, msg: Message, dialogue: SettingsDialogue) -> HandlerResult {
    let state = dialogue.get().await?;

    if state.is_none() {
        return Ok(());
    }

    if let Some(SettingsState::Settings { user_id } | SettingsState::Greeting { user_id }) = state {
        if user_id == msg.from().unwrap().id {
            let chat_id = msg.chat.id;
            let settings = settings::get(chat_id);
            let text = t!("settings-cancel", settings.language);
            let message = bot.send_message(chat_id, text).await?;
            let _ =
                utils::delete_message_later(&bot, chat_id, message.id, settings.message_expire());

            dialogue.exit().await?;
        }
    }

    Ok(())
}

async fn input_greeting_handler(
    bot: Bot,
    msg: Message,
    dialogue: SettingsDialogue,
    user_id: UserId,
) -> HandlerResult {
    if user_id != msg.from().unwrap().id {
        return Ok(());
    }

    let chat_id = msg.chat.id;
    let settings = settings::get(chat_id);
    let text = msg.text();

    if text.is_none() {
        let text = t!("settings-text-required", settings.language);
        bot.send_message(chat_id, text).await?;
        return Ok(());
    }

    let raw_greeting: Result<RawGreeting, _> = text.unwrap().parse();

    if let Err(error) = raw_greeting {
        let text = format!("Error parsing greeting: {error}.\nTry again or /cancel");
        bot.send_message(chat_id, text).await?;
        return Ok(());
    }

    let greeting = raw_greeting.unwrap().0;
    let text = t!(
        "settings-greeting-changed",
        settings.language,
        greeting = &greeting
    );

    // Send greeting back to check its validity.
    match bot.send_message(chat_id, text).await {
        Ok(message) => {
            settings::set_greeting(chat_id, greeting).await?;
            let _ =
                utils::delete_message_later(&bot, chat_id, message.id, settings.message_expire());
            dialogue.exit().await?;
        }
        Err(error) => {
            bot.send_message(chat_id, error.to_string()).await?;
        }
    }

    Ok(())
}

async fn help_command_handler(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .reply_to_message_id(msg.id)
        .await?;
    Ok(())
}

async fn new_chat_members_handler(
    bot: Bot,
    msg: Message,
    chat_members: Vec<User>,
    me: Me,
) -> HandlerResult {
    let chat_id = msg.chat.id;
    let users = chat_members.iter().filter(|m| !m.is_bot);

    // Restrict new users as soon as possible.
    let users_futures = users.clone().map(|user| {
        bot.restrict_chat_member(chat_id, user.id, ChatPermissions::empty())
            .into_future()
    });
    let restrictions = futures::future::join_all(users_futures).await;

    let settings = settings::get(chat_id);
    if settings.delete_entry_messages {
        tokio::spawn(bot.delete_message(chat_id, msg.id).into_future());
    }

    // Show error from the first failed restriction (if any).
    let failed_restrictions: Vec<_> = restrictions.iter().filter(|r| r.is_err()).collect();
    if failed_restrictions.len() > 0 {
        if let Err(error) = failed_restrictions[0] {
            let message = format!("Failed to restrict user: {error}");
            log::error!("{message}");
            bot.send_message(chat_id, message).await?;
        }
    }

    let bots = chat_members.iter().filter(|m| m.is_bot);
    for b in bots {
        if b.id == me.id {
            let text = t!("make-me-admin", settings.language);
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
            settings.language,
            user_tag = &user_tag,
            duration = settings.captcha_expire.get()
        );
        let message = bot
            .send_photo(chat_id, InputFile::url(url))
            .caption(caption)
            .reply_markup(keyboard)
            .await?;
        let text = t!(
            "captcha-time-over",
            settings.language,
            user_tag = user_tag,
            duration = settings.ignore_expire.get()
        );
        let _ = utils::delete_captcha_later(
            &bot,
            chat_id,
            message.id,
            text,
            settings.captcha_expire(),
            settings.message_expire(),
        );
        redis::set_answer(
            chat_id,
            user.id,
            comb.answer,
            settings.captcha_expire.get(),
            settings.ignore_expire.get(),
        )
        .await?;
    }

    Ok(())
}

async fn captcha_response_handler(bot: Bot, query: CallbackQuery) -> HandlerResult {
    if query.data.is_none() || query.message.is_none() {
        return Ok(());
    }

    let answer = query.data.unwrap();
    let message = query.message.unwrap();
    let chat_id = message.chat.id;
    let user_id = query.from.id;
    let settings = settings::get(chat_id);
    let correct_answer = redis::get_answer(chat_id, user_id).await?;

    if correct_answer.is_none() {
        let text = t!("query-wrong-user", settings.language);
        bot.answer_callback_query(query.id).text(text).await?;
        return Ok(());
    }

    let correct_answer = correct_answer.unwrap();
    let user_tag = html::user_mention_or_link(&query.from);

    if *answer == correct_answer {
        let text = t!("query-correct", settings.language);
        let (restriction, _, _) = tokio::join!(
            bot.restrict_chat_member(chat_id, user_id, ChatPermissions::all())
                .into_future(),
            bot.answer_callback_query(query.id).text(text).into_future(),
            bot.delete_message(chat_id, message.id).into_future()
        );

        // Show error if restriction didn't work.
        if let Err(error) = restriction {
            let message = format!("Failed to restrict user: {error}");
            log::error!("{message}");
            bot.send_message(chat_id, message).await?;
            return Ok(());
        }

        let text = if let Some(text) = settings::get_greeting(chat_id) {
            let mut vars = HashMap::new();
            vars.insert("user_tag".to_string(), user_tag);
            strfmt(&text, &vars).unwrap()
        } else {
            t!("greeting", settings.language, user_tag)
        };
        let message = bot.send_message(chat_id, text).await?;
        let _ = utils::delete_message_later(&bot, chat_id, message.id, settings.message_expire());
    } else {
        let text = t!("query-wrong", settings.language);
        let _ = tokio::join!(
            bot.answer_callback_query(query.id).text(text).into_future(),
            bot.delete_message(chat_id, message.id).into_future()
        );
        let text = t!(
            "captcha-incorrect-answer",
            settings.language,
            user_tag = user_tag,
            duration = settings.ignore_expire.get()
        );
        let message = bot.send_message(chat_id, text).await?;
        let _ = utils::delete_message_later(&bot, chat_id, message.id, settings.message_expire());
    }

    Ok(())
}
