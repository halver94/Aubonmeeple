use scraper::{Html, Selector};

pub async fn get_ludocortex_price_and_url_by_barcode(barcode: u64) -> Option<(f32, String)> {
    let search = format!("https://www.ludocortex.fr/jolisearch?s={}", barcode);
    log::debug!("[TASK] search on ludocortex by barcode: {}", &search);
    let content = reqwest::get(&search).await.unwrap().bytes().await.unwrap();
    let document = Html::parse_document(std::str::from_utf8(&content).unwrap());

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
        regular_price.as_ref()?;
        let regular_price = regular_price
            .unwrap()
            .trim()
            .replace("&nbsp;€", "")
            .replace(',', ".")
            .parse::<f32>();

        if href.is_none() || title.is_none() || regular_price.is_err() {
            return None;
        }

        if href.unwrap().contains(&barcode.to_string()) {
            return Some((regular_price.unwrap(), href.unwrap().to_string()));
        }
    }

    None
}

pub async fn get_ludocortex_price_and_url_by_name(name: &String) -> Option<(f32, String)> {
    let search = format!("https://www.ludocortex.fr/jolisearch?s={}", name);
    log::debug!("[TASK] search on ludocortex by name: {}", &search);

    let content = reqwest::get(&search).await.unwrap().bytes().await.unwrap();
    let document = Html::parse_document(std::str::from_utf8(&content).unwrap());

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
        regular_price.as_ref()?;
        let regular_price = regular_price
            .unwrap()
            .trim()
            .replace("&nbsp;€", "")
            .replace(',', ".")
            .parse::<f32>();

        if href.is_none() || title.is_none() || regular_price.is_err() {
            return None;
        }

        if title.unwrap().to_lowercase() == name.to_lowercase() {
            return Some((regular_price.unwrap(), href.unwrap().to_string()));
        }
    }

    None
}

pub async fn get_ludocortex_price_and_url(
    name: &String,
    barcode: Option<u64>,
) -> Option<(f32, String)> {
    if barcode.is_some() {
        if let Some((a, b)) = get_ludocortex_price_and_url_by_barcode(barcode.unwrap()).await {
            return Some((a, b));
        }
    }
    get_ludocortex_price_and_url_by_name(name).await
}
