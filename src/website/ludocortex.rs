use scraper::{Html, Selector};

use crate::website::helper::{are_names_similar, clean_name};

pub async fn get_ludocortex_price_and_url_by_barcode(
    barcode: u64,
) -> Result<Option<(f32, String)>, anyhow::Error> {
    let search = format!("https://www.ludocortex.fr/jolisearch?s={}", barcode);
    log::debug!("search on ludocortex by barcode: {}", barcode);
    let content = reqwest::get(&search).await?.bytes().await?;
    let document = Html::parse_document(std::str::from_utf8(&content)?);

    // Sélecteur pour l'article de produit
    let product_selector = Selector::parse(".product-miniature").unwrap();

    for product in document.select(&product_selector) {
        let href = product
            .select(&Selector::parse("a.product-thumbnail").unwrap())
            .next()
            .and_then(|link| link.value().attr("href"));

        let title = product
            .select(&Selector::parse(".product-title").unwrap())
            .next()
            .map(|title| title.inner_html());

        let regular_price = product
            .select(&Selector::parse(".regular-price").unwrap())
            .next()
            .map(|price| price.inner_html());

        if regular_price.is_none() {
            return Ok(None);
        }

        let regular_price = regular_price
            .unwrap()
            .trim()
            .replace("&nbsp;€", "")
            .replace(',', ".")
            .parse::<f32>();

        if href.is_none() || title.is_none() || regular_price.is_err() {
            LUDOCORTEX_STAT.with_label_values(&["fail"]).inc();
            return Ok(None);
        }

        if href.unwrap().contains(&barcode.to_string()) {
            LUDOCORTEX_STAT.with_label_values(&["success"]).inc();
            return Ok(Some((regular_price.unwrap(), href.unwrap().to_string())));
        }
    }

    LUDOCORTEX_STAT.with_label_values(&["fail"]).inc();
    Ok(None)
}

pub async fn get_ludocortex_price_and_url_by_name(
    name: &str,
) -> Result<Option<(f32, String)>, anyhow::Error> {
    let search = format!(
        "https://www.ludocortex.fr/jolisearch?s={}",
        clean_name(name)
    );
    log::debug!("search on ludocortex by name: {}", &name);

    let content = reqwest::get(&search).await?.bytes().await?;
    let document = Html::parse_document(std::str::from_utf8(&content)?);

    // Sélecteur pour l'article de produit
    let product_selector = Selector::parse(".product-miniature").unwrap();

    for product in document.select(&product_selector) {
        let href = product
            .select(&Selector::parse("a.product-thumbnail").unwrap())
            .next()
            .and_then(|link| link.value().attr("href"));

        let title = product
            .select(&Selector::parse(".product-title").unwrap())
            .next()
            .map(|title| title.inner_html());

        let regular_price = product
            .select(&Selector::parse(".regular-price").unwrap())
            .next()
            .map(|price| price.inner_html());

        if regular_price.is_none() {
            return Ok(None);
        }

        let regular_price = regular_price
            .unwrap()
            .trim()
            .replace("&nbsp;€", "")
            .replace(',', ".")
            .parse::<f32>();

        if href.is_none() || title.is_none() || regular_price.is_err() {
            LUDOCORTEX_STAT.with_label_values(&["success"]).inc();
            return Ok(None);
        }

        if are_names_similar(&(title.unwrap()), name) {
            LUDOCORTEX_STAT.with_label_values(&["success"]).inc();
            return Ok(Some((regular_price.unwrap(), href.unwrap().to_string())));
        }
    }

    LUDOCORTEX_STAT.with_label_values(&["fail"]).inc();
    Ok(None)
}

pub async fn get_ludocortex_price_and_url(
    name: &str,
    barcode: Option<u64>,
) -> Result<Option<(f32, String)>, anyhow::Error> {
    if barcode.is_some() {
        if let Some((a, b)) = get_ludocortex_price_and_url_by_barcode(barcode.unwrap()).await? {
            return Ok(Some((a, b)));
        }
    }
    get_ludocortex_price_and_url_by_name(name).await
}

use lazy_static::lazy_static;
use prometheus::{register_int_counter_vec, IntCounterVec};
lazy_static! {
    static ref LUDOCORTEX_STAT: IntCounterVec = register_int_counter_vec!(
        "ludocortex_stat",
        "Stat about parsing/fetch success/fail for this website",
        &["result"]
    )
    .unwrap();
}
