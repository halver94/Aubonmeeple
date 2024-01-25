use hyper::StatusCode;
use reqwest::{Client, ClientBuilder, IntoUrl, Response};
use scraper::Html;

lazy_static::lazy_static! {
    static ref CLIENT: Client = create_client();
}

fn create_client() -> Client {
    ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to build reqwest::Client")
}

/// Execute a request using the shared http client
pub async fn get<U: IntoUrl>(url: U) -> Result<Response, reqwest::Error> {
   CLIENT.get(url).send().await
}

/// Fetch an HTML document from a URL
pub async fn get_doc<U: IntoUrl>(url: U) -> Result<(Html, StatusCode), reqwest::Error> {
    let response = CLIENT.get(url).send().await?;
    let http_code = response.status();

    let content = response.text().await?;
    let document = Html::parse_document(&content);
    Ok((document, http_code))
}