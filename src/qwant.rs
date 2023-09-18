use rand::{seq::SliceRandom, thread_rng};
use reqwest::{header, Client, Url};
use serde_json::Value;
use std::fmt;

const SEARCH_URL: &str = "https://api.qwant.com/v3/search/images";
const PARAMS: [(&str, &str); 6] = [
    ("count", "10"),
    ("t", "images"),
    ("safesearch", "1"),
    ("locale", "en_US"),
    ("offset", "0"),
    ("device", "desktop"),
];
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:102.0) Gecko/20100101 Firefox/102.0";

#[derive(Debug)]
struct QwantResponseError;

impl fmt::Display for QwantResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "something wrong with Qwant response")
    }
}

impl std::error::Error for QwantResponseError {}

pub async fn get_image_url(query: &str) -> Result<Url, crate::Error> {
    log::info!("Searching images on Qwant with query: \"{query}\"");
    let mut url = Url::parse_with_params(SEARCH_URL, PARAMS).unwrap();
    url.query_pairs_mut().append_pair("q", query);
    let client = Client::new();
    let data: Value = client
        .get(url)
        .header(header::USER_AGENT, USER_AGENT)
        .header(header::ACCEPT, "application/json")
        .send()
        .await?
        .json()
        .await?;
    if let Value::Array(items) = &data["data"]["result"]["items"] {
        let mut rng = thread_rng();
        if let Value::Object(map) = items.choose(&mut rng).unwrap() {
            if let Value::String(url) = map.get("thumbnail").unwrap() {
                return Ok(Url::parse(url)?);
            }
        }
    }

    Err(Box::new(QwantResponseError))
}
