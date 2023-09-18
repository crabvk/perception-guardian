use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, MessageId, True};
use teloxide::{prelude::Requester, requests::ResponseResult};
use tokio::{
    task::JoinHandle,
    time::{sleep, Duration},
};

pub fn emojis_keyboard(emojis: &[&str], rows: usize) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::with_capacity(rows);
    let row_size = (emojis.len() as f64 / rows as f64).ceil() as usize;

    for row in emojis.chunks(row_size) {
        let kb_row = row
            .iter()
            .map(|emoji| InlineKeyboardButton::callback(emoji.to_owned(), emoji.to_owned()))
            .collect();

        keyboard.push(kb_row);
    }

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard(pairs: &[(&str, &str)], row_size: usize) -> InlineKeyboardMarkup {
    let rows = (pairs.len() as f64 / row_size as f64).ceil() as usize;
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::with_capacity(rows);

    for row in pairs.chunks(row_size) {
        let kb_row = row
            .iter()
            .map(|(title, idx)| InlineKeyboardButton::callback(title.to_owned(), idx.to_string()))
            .collect();

        keyboard.push(kb_row);
    }

    InlineKeyboardMarkup::new(keyboard)
}

pub fn delete_message_later(
    bot: &crate::Bot,
    chat_id: ChatId,
    message_id: MessageId,
    timeout: Duration,
) -> JoinHandle<ResponseResult<True>> {
    let bot = bot.clone();
    tokio::spawn(async move {
        sleep(timeout).await;
        bot.delete_message(chat_id, message_id).await
    })
}

pub fn delete_captcha_later(
    bot: &crate::Bot,
    chat_id: ChatId,
    captcha_message_id: MessageId,
    text: String,
    captcha_timeout: Duration,
    service_message_timeout: Duration,
) -> JoinHandle<()> {
    let bot = bot.clone();
    tokio::spawn(async move {
        // Wait while CAPTCHA becomes expired, delete it and show temporary service message.
        sleep(captcha_timeout).await;
        let result = bot.delete_message(chat_id, captcha_message_id).await;
        // Show service message only when CAPTCHA was deleted.
        if result.is_ok() {
            if let Ok(message) = bot.send_message(chat_id, text).await {
                sleep(service_message_timeout).await;
                let _ = bot.delete_message(chat_id, message.id).await;
            }
        }
    })
}
