use std::num::NonZeroU32;
use hyper::StatusCode;
use reqwest::{Client, ClientBuilder, IntoUrl, Response};
use scraper::Html;
use governor::{Quota, RateLimiter, DefaultKeyedRateLimiter, clock};
use governor::state::keyed::DefaultKeyedStateStore;
use nonzero_ext::nonzero;

/// DEFAULT_QUOTA is the default requests per minutes if not specified
const DEFAULT_QUOTA: Quota = Quota::per_minute(nonzero!(30u32));

lazy_static::lazy_static! {
    static ref CLIENT: Client = create_client();
    static ref LIMITER_CLOCK: clock::DefaultClock = clock::DefaultClock::default();
    static ref LIMITER: DefaultKeyedRateLimiter<String> = create_limiter();
}

fn create_client() -> Client {
    ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to build reqwest::Client")
}

fn create_limiter() -> DefaultKeyedRateLimiter<String> {
    let quota = std::env::var("RATELIMIT_PER_MINUTE")
        .map_err(|v| v.to_string())
        .and_then(|v| v.parse::<u32>().map_err(|v| v.to_string()))
        .and_then(|v| NonZeroU32::new(v).ok_or_else(|| "Cannot convert to non-zero u32".to_string()))
        .map(Quota::per_minute)
        .unwrap_or_else(|err| {
            log::warn!("Cannot initialize quota from environment, fallback to default: {}", err);
            DEFAULT_QUOTA
        })
        .allow_burst(nonzero!(1u32));

    log::debug!("Creating rate limiter with quota {:?}", quota);
    let state = DefaultKeyedStateStore::default();
    RateLimiter::new(quota, state, &LIMITER_CLOCK)
}


/// Execute a request using the shared http client
pub async fn get<U: IntoUrl>(url: U) -> Result<Response, reqwest::Error> {
    let url_r = url.into_url()?;

    let ratelimit_key = url_r.host_str().unwrap().to_string();
    log::debug!("get_doc {}", url_r);
    LIMITER.until_key_ready(&ratelimit_key).await;

    CLIENT.get(url_r).send().await
}

/// Fetch an HTML document from a URL
/// The requests are rate-limited by host
pub async fn get_doc<U: IntoUrl>(url: U) -> Result<(Html, StatusCode), reqwest::Error> {
    let response = get(url).await?;
    let http_code = response.status();

    let content = response.text().await?;
    let document = Html::parse_document(&content);
    Ok((document, http_code))
}