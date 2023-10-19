use scraper::{Html, Selector};

use crate::website::helper::are_names_similar;

pub async fn get_agorajeux_price_and_url_by_name(name: &str) -> Option<(f32, String)> {
    let name_clean = normalize_agorajeux_name(name);
    let search = format!(
        "https://www.agorajeux.com/fr/recherche?controller=search&s={}",
        name_clean
    );
    log::debug!("[TASK] search on agorajeux: {}", &search);
    let content = reqwest::get(&search).await.unwrap().bytes().await.unwrap();
    parse_agorajeux_document(name, &std::str::from_utf8(&content).unwrap())
}

fn normalize_agorajeux_name(name: &str) -> String {
    name.replace('&', " ")
}

fn parse_agorajeux_document(name: &str, doc: &str) -> Option<(f32, String)> {
    let document = Html::parse_document(doc);

    let product_selector = Selector::parse(".js-product-miniature").unwrap();
    let href_selector = Selector::parse("a.thumbnail.product-thumbnail").unwrap();
    let price_selector = Selector::parse(".product-price-and-shipping .price").unwrap();
    let product_name_selector = Selector::parse("span.h3.product-title a").unwrap();

    log::trace!("Parsing agorajeux document");
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
    None
}

#[cfg(test)]
mod tests {
    use super::{normalize_agorajeux_name, parse_agorajeux_document};
    use log::Level;
    use std::{env, fs};

    struct Test {
        name: String,
        price: f32,
        href: String,
        document: String,
    }

    #[test]
    fn test_parsing() {
        env::set_var("RUST_LOG", "boardgame_finder=trace");
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or(Level::Info.as_str()),
        )
        .try_init();

        let tests =
            vec![
            Test {
            price: 26.5,
            href: "https://www.agorajeux.com/fr/jeux-d-enquetes/14624-break-in-tour-eiffel.html".to_string(),
            name: "Break In - Tour Eiffel".to_string(),
            document: "tests/agorajeux/test1.html".to_string(),
            },
            Test {
            price: 22.41,
            href: "https://www.agorajeux.com/fr/les-jeux-pour-toute-la-famille/11063-my-little-scythe-le-gateau-dans-le-ciel.html".to_string(),
            name: "My Little Scythe - Le Gâteau Dans Le Ciel".to_string(),
            document: "tests/agorajeux/test2.html".to_string(),
            },
            Test {
            price: 23.90,
            href: "https://www.agorajeux.com/fr/jeux-gigamic/3047-quarto-mini.html".to_string(),
            name: "Quarto! Mini".to_string(),
            document: "tests/agorajeux/test3.html".to_string(),
            },
            Test {
            price: 24.90,
            href: "https://www.agorajeux.com/fr/jeux-d-enquetes/13407-death-note-le-jeu-d-enquete.html".to_string(),
            name: "Death Note Le Jeu D'enquête".to_string(),
            document: "tests/agorajeux/test4.html".to_string(),
            },
        ];
        for test in tests.into_iter() {
            let name_clean = normalize_agorajeux_name(&test.name);
            let doc =
                fs::read_to_string(test.document).expect("Should have been able to read the file");
            if let Some((price, href)) = parse_agorajeux_document(&(name_clean), &doc) {
                assert_eq!(price, test.price);
                assert_eq!(href, test.href);
            } else {
                panic!("fail to parse");
            }
        }
    }
}
