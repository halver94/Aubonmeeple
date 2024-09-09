use governor::state::keyed::DefaultKeyedStateStore;
use governor::{clock, DefaultKeyedRateLimiter, Quota, RateLimiter};
use hyper::{HeaderMap, StatusCode};
use nonzero_ext::nonzero;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use reqwest::{Client, ClientBuilder, IntoUrl, Response};
use scraper::Html;
use std::num::NonZeroU32;
use std::time::Duration;
use std::{env, fs};

/// DEFAULT_QUOTA is the default requests per minutes if not specified
const DEFAULT_QUOTA: Quota = Quota::per_minute(nonzero!(60u32));

lazy_static::lazy_static! {
    pub static ref CLIENTS: Vec<Client> = create_client();
    static ref LIMITER_CLOCK: clock::DefaultClock = clock::DefaultClock::default();
    static ref LIMITER: DefaultKeyedRateLimiter<String> = create_limiter();
}

//reqwest::Proxy::all("socks5://23.19.244.109:1080");
pub fn create_client() -> Vec<Client> {
    log::debug!("init https client");

    let connection_timeout = Duration::from_secs(15);

    let mut headers = HeaderMap::new();
    headers.insert("Connection", "keep-alive".parse().unwrap());
    headers.insert(
        "User-Agent",
        "Mozilla/5.0 (X11; Linux x86_64; rv:102.0) Gecko/20100101 Firefox/102.0"
            .parse()
            .unwrap(),
    );

    let mut clients = vec![];
    let filename = env::var("PROXY_FILE").unwrap_or("fichier.txt".to_string());
    match fs::read_to_string(filename) {
        Ok(content) => {
            for proxy in content.lines() {
                log::debug!("got proxy {:?}", proxy);
                let proxy = reqwest::Proxy::all(format!("socks5h://{}", proxy)).unwrap();
                let client = ClientBuilder::new()
                    .proxy(proxy)
                    .timeout(connection_timeout)
                    .danger_accept_invalid_certs(true)
                    .default_headers(headers.clone())
                    .redirect(reqwest::redirect::Policy::none())
                    .build()
                    .expect("Failed to build reqwest::Client");
                clients.push(client);
            }
        }
        Err(e) => {
            log::error!("Cannot read PROXY_FILE path, no proxy used: {}", e);
            let client = ClientBuilder::new()
                .timeout(connection_timeout)
                .danger_accept_invalid_certs(true)
                .default_headers(headers)
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("Failed to build reqwest::Client");
            clients.push(client);
        }
    }
    clients
}

fn create_limiter() -> DefaultKeyedRateLimiter<String> {
    let quota = std::env::var("RATELIMIT_PER_MINUTE")
        .map_err(|v| v.to_string())
        .and_then(|v| v.parse::<u32>().map_err(|v| v.to_string()))
        .and_then(|v| {
            NonZeroU32::new(v).ok_or_else(|| "Cannot convert to non-zero u32".to_string())
        })
        .map(Quota::per_minute)
        .unwrap_or_else(|err| {
            log::warn!(
                "Cannot initialize quota from environment, fallback to default: {}",
                err
            );
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

    let mut rng: StdRng = SeedableRng::from_entropy();
    loop {
        let client = CLIENTS.get(rng.gen_range(0..CLIENTS.len())).unwrap();
        log::debug!("Sending request to {:?}", client);
        match client.get(url_r.clone()).send().await {
            Ok(resp) => {
                return Ok(resp);
            }
            Err(e) => {
                log::error!("error sending request to {:?} : {}", client, e);
                continue;
            }
        }
    }
}

/// Fetch an HTML document from a URL
/// The requests are rate-limited by host
pub async fn get_doc<U: IntoUrl>(url: U) -> Result<(Html, StatusCode), reqwest::Error> {
    let response = get(url).await?;
    let http_code = response.status();

    let content = response.text().await?;

    //let content = response.text().await?;
    let document = Html::parse_document(&content);
    Ok((document, http_code))
}
