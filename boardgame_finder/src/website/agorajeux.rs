use scraper::{Html, Selector};

pub async fn get_agorajeux_price_and_url_by_name(name: &str) -> Option<(f32, String)> {
    let search = format!(
        "https://www.agorajeux.com/fr/recherche?controller=search&s={}",
        name.replace(' ', "+")
    );
    println!("Search on agorajeux: {}", &search);
    let content = reqwest::get(&search).await.unwrap().bytes().await.unwrap();
    let document = Html::parse_document(std::str::from_utf8(&content).unwrap());

    let product_selector = Selector::parse(".js-product-miniature").unwrap();
    let href_selector = Selector::parse("a.thumbnail.product-thumbnail").unwrap();
    let price_selector = Selector::parse(".regular-price").unwrap();
    let product_name_selector = Selector::parse("span.h3.product-title a").unwrap();

    for product in document.select(&product_selector) {
        let href_element = product.select(&href_selector).next();
        if let Some(href) = href_element {
            let href_attr = href.value().attr("href").unwrap_or_default();

            let price_element = product.select(&price_selector).next();
            if let Some(price) = price_element {
                let price_text = price.text().collect::<String>();
                let mut price = price_text.trim().replace(',', ".");
                // this is ugly but I dont know why others technics dont work to remove the last 2 chars here
                price.pop();
                price.pop();
                let price = price.parse::<f32>().unwrap_or(0.0);

                let product_name_element = product.select(&product_name_selector).next();
                if let Some(product_name) = product_name_element {
                    let product_name_text = product_name.text().collect::<String>();
                    let product_name_text = product_name_text.trim();

                    println!(
                        "Product Name: {}, Price: {}, Href: {}",
                        product_name_text, price_text, href_attr
                    );
                    if product_name_text.to_lowercase() == name.to_lowercase() {
                        return Some((price, href_attr.to_string()));
                    }
                }
            }
        }
    }
    None
}
