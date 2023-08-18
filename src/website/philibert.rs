use log::debug;
use scraper::{Html, Selector};

pub async fn get_philibert_price_and_url_by_barcode(barcode: u64) -> Option<(f32, String)> {
    let search = format!(
        "https://www.philibertnet.com/fr/recherche?search_query={}&submit_search=",
        barcode
    );
    debug!("Search on philibert: {}", &search);
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
                    debug!("href : {} , barcode : {}", href_attr, barcode);
                    return Some((price_text, href_attr.to_string()));
                }
            }
        }
    }
    None
}

pub async fn get_philibert_price_and_url_by_name(name: &str) -> Option<(f32, String)> {
    let search = format!(
        "https://www.philibertnet.com/fr/recherche?search_query={}&submit_search=",
        name
    );
    debug!("Search on philibert: {}", &search);
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

                if title_text.to_lowercase() == name.to_lowercase() {
                    return Some((price_text, href_attr.to_string()));
                }
            }
        }
    }
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
