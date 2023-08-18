use feed_rs::{
    model::Feed,
    parser::{self, ParseFeedError},
};
use scraper::{Html, Selector};

pub async fn get_okkazeo_barcode_and_city(id: u32) -> (Option<u64>, Option<String>) {
    let search = format!("https://www.okkazeo.com/annonces/view/{}", id);
    println!("Getting city and barcode from okkazeo : {}", &search);
    let content = reqwest::get(search).await.unwrap().bytes().await.unwrap();
    let document = Html::parse_document(std::str::from_utf8(&content).unwrap());

    let barcode_selector = Selector::parse("i.fa-barcode").unwrap();
    let city_selector = Selector::parse("div.gray div.grid-x div.cell").unwrap();

    let barcode = if let Some(barcode) = document.select(&barcode_selector).next() {
        barcode
            .next_sibling()
            .and_then(|node| node.value().as_text())
            .map(|text| text.trim())
            .map(|text| text.parse::<u64>().unwrap_or(0))
    } else {
        None
    };

    if let Some(city_element) = document.select(&city_selector).next() {
        let city = city_element.text().collect::<Vec<_>>().join("");
        println!("barcode {:#?}, city {:#?}", barcode, city);
        return (barcode, Some(city));
    };

    (barcode, None)
}

pub async fn get_atom_feed() -> Result<Feed, ParseFeedError> {
    let content = reqwest::get("https://www.okkazeo.com/annonces/atom/0/50")
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    parser::parse(content.as_ref())
}
