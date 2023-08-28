use std::{
    collections::HashMap,
    fs::File,
    io::{Cursor, Write},
    path::Path,
};

use feed_rs::{
    model::Feed,
    parser::{self, ParseFeedError},
};
use hyper::StatusCode;
use image::io::Reader as ImageReader;
use regex::Regex;
use reqwest::{get, Client, ClientBuilder};
use scraper::{Html, Selector};

use crate::game::Seller;

pub async fn game_still_available(id: u32) -> bool {
    log::debug!("[TASK] checking if game with id {} is still available", id);
    let (_, code) = get_okkazeo_announce_page(id).await;

    log::debug!(
        "[TASK] game {} still available, is redirection: {} : code http {}",
        id,
        code.is_redirection(),
        code
    );
    !code.is_redirection()
}

pub fn okkazeo_is_pro_seller(document: &Html) -> bool {
    log::debug!("[TASK] checking if seller is pro");

    let class_selector = Selector::parse("i.fas.fa-fw.fa-user-tie.big").unwrap();

    let is_pro_present = document.select(&class_selector).next().is_some();

    if is_pro_present {
        log::debug!("[TASK] The 'PRO' tag is present.");
    } else {
        log::debug!("[TASK] The 'PRO' tag is not present.");
    }

    is_pro_present
}

pub fn get_okkazeo_shipping(document: &Html) -> HashMap<String, f32> {
    log::debug!("[TASK] getting shipping from okkazeo");

    let mut ships = HashMap::<String, f32>::new();
    // Vérifier la présence de 'handshake'
    let handshake_selector = Selector::parse("i.far.fa-fw.fa-handshake").unwrap();
    let is_handshake_present = document.select(&handshake_selector).next().is_some();

    if is_handshake_present {
        ships.insert("hand_delivery".to_string(), 0.0);
    }

    // Extraire les modes d'expédition
    let truck_selector = Selector::parse("div.cell.small-8.large-3").unwrap();
    let price_selector = Selector::parse("div.cell.small-4.large-1.text-right").unwrap();

    for (truck, price) in document
        .select(&truck_selector)
        .zip(document.select(&price_selector))
    {
        let truck_name = truck.text().collect::<Vec<_>>().join("").trim().to_string();
        let truck_price = price
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string()
            .replace(",", ".")
            .replace(" €", "")
            .parse::<f32>()
            .unwrap_or_default();

        ships.insert(truck_name, truck_price);
    }

    log::debug!("[TASK] shipping :{:#?}", ships);
    ships
}

pub fn get_okkazeo_seller(document: &Html) -> Option<Seller> {
    log::debug!("[TASK] getting seller from okkazeo");

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

    log::debug!(
        "[TASK] seller: {}, href: {}, nb Annonces: {}",
        seller_name,
        href_attr,
        nb_annonces_text
    );
    Some(Seller {
        name: seller_name.to_string(),
        url: format!(
            "https://www.okkazeo.com/{}",
            href_attr.to_string().replace("viewProfil", "stock")
        ),
        nb_announces: nb_annonces_text,
        is_pro: false,
    })
}

pub fn get_okkazeo_barcode(document: &Html) -> Option<u64> {
    log::debug!("[TASK] getting barcode from okkazeo");

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

pub fn get_okkazeo_city(document: &Html) -> Option<String> {
    log::debug!("[TASK] getting city from okkazeo");

    let city_selector = Selector::parse("div.gray div.grid-x div.cell").unwrap();

    if let Some(city_element) = document.select(&city_selector).next() {
        let city = city_element.text().collect::<Vec<_>>().join("");
        return Some(city);
    };

    None
}

pub async fn get_okkazeo_announce_page(id: u32) -> (Html, StatusCode) {
    let search = format!("https://www.okkazeo.com/annonces/view/{}", id);
    log::debug!("[TASK] getting announce page from okkazeo : {}", &search);

    let client = ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let response = client.get(search).send().await.unwrap();

    let http_code = response.status();
    let content = response.bytes().await.unwrap();
    let document = Html::parse_document(std::str::from_utf8(&content).unwrap());
    (document, http_code)
}

pub async fn get_okkazeo_game_image(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    log::debug!("[TASK] getting image from {}", url);
    let response = get(url).await?;
    let image_bytes = response.bytes().await?;

    // Créer un lecteur d'image à partir des données téléchargées
    let image_reader = ImageReader::new(std::io::Cursor::new(image_bytes))
        .with_guessed_format()
        .expect("Failed to guess image format");

    // Lire l'image depuis le lecteur
    let image = image_reader.decode().unwrap();

    // Convertissez l'image en PNG (vous pouvez également utiliser JPEG)
    let mut bytes: Vec<u8> = Vec::new();
    image.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)?;

    let re = Regex::new(r#"/([^/]+)\.(jpg|png)$"#).unwrap();
    let mut name: &str = "unknown";
    if let Some(captures) = re.captures(url) {
        if let Some(filename) = captures.get(1) {
            name = filename.as_str();
        }
    }

    // Enregistrez l'image convertie sur le disque
    if !std::path::Path::new("img").exists() {
        std::fs::create_dir("img").unwrap();
    }
    let output_path = Path::new("img").join(format!("{}{}", name, ".png"));
    let mut output_file = File::create(&output_path).unwrap();
    output_file.write_all(&bytes).unwrap();

    Ok(output_path.to_str().unwrap().to_string())
}

pub async fn get_atom_feed() -> Result<Feed, ParseFeedError> {
    log::debug!("[TASK] getting atom feed");
    let content = reqwest::get("https://www.okkazeo.com/annonces/atom/0/50")
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    parser::parse(content.as_ref())
}
