use scraper::{Html, Selector};

use crate::httpclient;
use crate::website::helper::are_names_similar;

pub async fn get_ludifolie_price_and_url_by_name(
    name: &str,
) -> Result<Option<(f32, String)>, anyhow::Error> {
    let name_clean = normalize_ludifolie_name(name);
    let search = format!(
        "https://www.ludifolie.com/recherche?controller=search&s={}",
        name_clean
    );
    log::debug!(
        "search on ludifolie: {} , cleaned_name : {}",
        &name,
        name_clean
    );

    let (doc, _) = httpclient::get_doc(&search).await?;
    Ok(parse_ludifolie_document(name, &doc))
}

fn normalize_ludifolie_name(name: &str) -> String {
    name.replace('&', " ")
}

fn parse_ludifolie_document(name: &str, document: &Html) -> Option<(f32, String)> {
    let product_selector = Selector::parse(".product-miniature-wrapper").unwrap();
    let href_selector = Selector::parse(".product-title a").unwrap();
    let price_selector = Selector::parse(".product-price-and-shipping .price").unwrap();
    let product_name_selector = Selector::parse(".product-title a").unwrap();

    log::trace!("parsing ludifolie document for {}", name);
    for product in document.select(&product_selector) {
        let href_element = product.select(&href_selector).next();
        if let Some(href) = href_element {
            let href_attr = href.value().attr("href").unwrap_or_default();
            log::trace!("href : {}", href_attr);

            let price_element = product.select(&price_selector).next();
            if let Some(price) = price_element {
                let price_text = price.text().collect::<String>();
                let mut price = price_text.trim().replace(',', ".");
                // this is ugly but I dont know why others technics dont work to remove the last 2 chars here
                price.pop();
                price.pop();
                log::trace!("price : {}", price);
                let price = price.parse::<f32>().unwrap_or(0.0);

                let product_name_element = product.select(&product_name_selector).next();
                if let Some(product_name) = product_name_element {
                    let processed_name = product_name.text().collect::<String>();
                    if are_names_similar(processed_name.as_str(), name) {
                        LUDIFOLIE_STAT.with_label_values(&["success"]).inc();
                        return Some((price, href_attr.to_string()));
                    }
                }
            } else {
                log::trace!("fail to select price");
            }
        } else {
            log::trace!("fail to select href");
        }
    }
    LUDIFOLIE_STAT.with_label_values(&["fail"]).inc();
    None
}

use lazy_static::lazy_static;
use prometheus::{register_int_counter_vec, IntCounterVec};
lazy_static! {
    static ref LUDIFOLIE_STAT: IntCounterVec = register_int_counter_vec!(
        "ludifolie_stat",
        "Stat about parsing/fetch success/fail for this website",
        &["result"]
    )
    .unwrap();
}
