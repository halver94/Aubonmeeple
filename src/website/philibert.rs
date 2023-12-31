use scraper::{Html, Selector};

use crate::website::helper::{are_names_similar, clean_name};

pub async fn get_philibert_price_and_url_by_barcode(barcode: u64) -> Option<(f32, String)> {
    let search = format!(
        "https://www.philibertnet.com/fr/recherche?search_query={}&submit_search=",
        barcode
    );
    log::debug!("[TASK] search on philibert by barcode: {}", &search);
    let content = reqwest::get(&search).await.unwrap().bytes().await.unwrap();
    let document = Html::parse_document(std::str::from_utf8(&content).unwrap());

    let product_list_selector = Selector::parse(".product_list.grid .ajax_block_product").unwrap();
    let price_selector = Selector::parse(".price").unwrap();
    let title_selector = Selector::parse("p.s_title_block a").unwrap();

    for product in document.select(&product_list_selector) {
        let price_element = product.select(&price_selector).next();
        if let Some(price) = price_element {
            let price_text = price.text().collect::<String>();
            let price_text = price_text
                .trim()
                .replace(" €", "")
                .replace(',', ".")
                .parse::<f32>()
                .unwrap_or(0.0);

            let title_element = product.select(&title_selector).next();
            if let Some(title) = title_element {
                let href_attr = title.value().attr("href").unwrap_or_default();

                if href_attr
                    .split('?')
                    .next()
                    .unwrap()
                    .contains(&barcode.to_string())
                {
                    PHILIBERT_STAT.with_label_values(&["success"]).inc();
                    return Some((price_text, href_attr.to_string()));
                }
            }
        }
    }
    PHILIBERT_STAT.with_label_values(&["fail"]).inc();
    None
}

pub async fn get_philibert_price_and_url_by_name(name: &str) -> Option<(f32, String)> {
    let search = format!(
        "https://www.philibertnet.com/fr/recherche?search_query={}&submit_search=",
        clean_name(name)
    );
    log::debug!("[TASK] search on philibert by name: {}", &search);
    let content = reqwest::get(&search).await.unwrap().bytes().await.unwrap();
    let document = Html::parse_document(std::str::from_utf8(&content).unwrap());

    let product_list_selector = Selector::parse(".product_list.grid .ajax_block_product").unwrap();
    let price_selector = Selector::parse(".price").unwrap();
    let title_selector = Selector::parse("p.s_title_block a").unwrap();

    for product in document.select(&product_list_selector) {
        let price_element = product.select(&price_selector).next();
        if let Some(price) = price_element {
            let price_text = price.text().collect::<String>();
            let price_text = price_text
                .trim()
                .replace(" €", "")
                .replace(',', ".")
                .parse::<f32>()
                .unwrap_or(0.0);

            let title_element = product.select(&title_selector).next();
            if let Some(title) = title_element {
                let title_text = title.text().collect::<String>();
                let title_text = title_text.trim();
                let href_attr = title.value().attr("href").unwrap_or_default();

                if are_names_similar(title_text, name) {
                    PHILIBERT_STAT.with_label_values(&["success"]).inc();
                    return Some((price_text, href_attr.to_string()));
                }
            }
        }
    }
    PHILIBERT_STAT.with_label_values(&["fail"]).inc();
    None
}

pub async fn get_philibert_price_and_url(
    name: &str,
    barcode: Option<u64>,
) -> Option<(f32, String)> {
    if barcode.is_some() {
        if let Some((a, b)) = get_philibert_price_and_url_by_barcode(barcode.unwrap()).await {
            return Some((a, b));
        }
    }
    get_philibert_price_and_url_by_name(name).await
}

use lazy_static::lazy_static;
use prometheus::{register_int_counter_vec, IntCounterVec};
lazy_static! {
    static ref PHILIBERT_STAT: IntCounterVec = register_int_counter_vec!(
        "philibert_stat",
        "Stat about parsing/fetch success/fail for this website",
        &["result"]
    )
    .unwrap();
}
