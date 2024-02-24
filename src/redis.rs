use redis::{aio::ConnectionManager, AsyncCommands, RedisResult};
use std::time::{Duration, SystemTime};
use teloxide::types::{ChatId, UserId};
use tokio::sync::OnceCell;

const ANSWER_PREFIX: &str = "answer";
const IGNORE_KEY: &str = "ignore";
static REDIS: OnceCell<ConnectionManager> = OnceCell::const_new();

pub async fn setup(url: url::Url) -> RedisResult<()> {
    let cm = ::redis::Client::open(url)
        .unwrap()
        .get_connection_manager()
        .await?;

    if REDIS.set(cm).is_err() {
        panic!("Couldn't set REDIS cell.");
    }

    Ok(())
}

pub async fn set_answer(
    chat_id: ChatId,
    user_id: UserId,
    answer: &str,
    captcha_expire: u64,
    ignore_expire: u64,
) -> RedisResult<()> {
    let key = format!("{ANSWER_PREFIX}:{chat_id}:{user_id}");
    let member = format!("{chat_id}:{user_id}");
    let ignore_expire = SystemTime::now() + Duration::from_secs(ignore_expire);
    let epoch = ignore_expire
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    log::info!("Ignore user {user_id} in chat {chat_id} until {epoch}");
    let mut cm = REDIS.get().unwrap().clone();
    redis::pipe()
        .atomic()
        .set(&key, answer)
        .expire(&key, captcha_expire as i64)
        .zadd(IGNORE_KEY, member, epoch)
        .query_async(&mut cm)
        .await?;

    Ok(())
}

pub async fn get_answer(chat_id: ChatId, user_id: UserId) -> RedisResult<Option<String>> {
    let key = format!("{ANSWER_PREFIX}:{chat_id}:{user_id}");
    let mut cm = REDIS.get().unwrap().clone();
    let answer = cm.get(key).await?;
    Ok(answer)
}

pub async fn is_ignored(chat_id: ChatId, user_id: UserId) -> bool {
    let member = format!("{chat_id}:{user_id}");
    let mut cm = REDIS.get().unwrap().clone();
    let result: RedisResult<Option<f64>> = cm.zscore(IGNORE_KEY, member).await;
    if let Ok(epoch) = result {
        if let Some(epoch) = epoch {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            return now < epoch;
        }
    }
    false
}
