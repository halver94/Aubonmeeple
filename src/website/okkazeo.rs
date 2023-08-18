use feed_rs::{
    model::Feed,
    parser::{self, ParseFeedError},
};
use scraper::{Html, Selector};

use crate::game::Seller;

pub async fn get_okkazeo_seller(document: &Html) -> Option<Seller> {
    println!("Getting seller from okkazeo");

    let seller_selector = Selector::parse(".seller").unwrap();
    let href_selector = Selector::parse(".div-seller").unwrap();
    let nb_annonces_selector = Selector::parse(".nb_annonces").unwrap();

    let seller = document.select(&seller_selector).next()?;
    let seller_name = seller.text().collect::<String>();
    let seller_name = seller_name.trim();

    let href_element = document.select(&href_selector).next()?;
    let href_attr = href_element.value().attr("href").unwrap_or_default();

    let nb_annonces_element = document.select(&nb_annonces_selector).next()?;
    let nb_annonces_text = nb_annonces_element
        .text()
        .collect::<String>()
        .trim()
        .parse::<u32>()
        .unwrap();

    println!(
        "Seller: {}, Href: {}, Nb Annonces: {}",
        seller_name, href_attr, nb_annonces_text
    );
    return Some(Seller {
        name: seller_name.to_string(),
        url: format!(
            "https://www.okkazeo.com/{}",
            href_attr.to_string().replace("viewProfil", "stock")
        ),
        nb_announces: nb_annonces_text,
    });
}

pub async fn get_okkazeo_barcode(document: &Html) -> Option<u64> {
    println!("Getting barcode from okkazeo");

    let barcode_selector = Selector::parse("i.fa-barcode").unwrap();
    let barcode = if let Some(barcode) = document.select(&barcode_selector).next() {
        barcode
            .next_sibling()
            .and_then(|node| node.value().as_text())
            .map(|text| text.trim())
            .map(|text| text.parse::<u64>().unwrap_or(0))
    } else {
        None
    };

    barcode
}

pub async fn get_okkazeo_city(document: &Html) -> Option<String> {
    println!("Getting city from okkazeo");

    let city_selector = Selector::parse("div.gray div.grid-x div.cell").unwrap();

    if let Some(city_element) = document.select(&city_selector).next() {
        let city = city_element.text().collect::<Vec<_>>().join("");
        return Some(city);
    };

    None
}

pub async fn get_okkazeo_announce_page(id: u32) -> Html {
    let search = format!("https://www.okkazeo.com/annonces/view/{}", id);
    println!("Getting city and barcode from okkazeo : {}", &search);
    let content = reqwest::get(search).await.unwrap().bytes().await.unwrap();
    let document = Html::parse_document(std::str::from_utf8(&content).unwrap());
    document
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
