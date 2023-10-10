use std::{
    collections::HashMap,
    error::{self},
    fs::File,
    io::{Cursor, Write},
    path::Path,
};

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use feed_rs::{
    model::Feed,
    parser::{self, ParseFeedError},
};
use hyper::StatusCode;
use image::io::Reader as ImageReader;
use regex::Regex;
use reqwest::{get, ClientBuilder};
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
    let class_selector = Selector::parse("i.fas.fa-fw.fa-user-tie.big").unwrap();

    document.select(&class_selector).next().is_some()
}

pub fn get_okkazeo_shipping(document: &Html) -> HashMap<String, f32> {
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

    ships
}

pub fn get_okkazeo_seller(document: &Html) -> Option<Seller> {
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

    let id = href_attr
        .split('/')
        .collect::<Vec<&str>>()
        .last()?
        .parse::<u32>()
        .unwrap();

    Some(Seller {
        id: id,
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
    let city_selector = Selector::parse("div.gray div.grid-x div.cell").unwrap();

    if let Some(city_element) = document.select(&city_selector).next() {
        let city = city_element.text().collect::<Vec<_>>().join("");
        return Some(city);
    };

    None
}

pub fn get_okkazeo_announce_price(
    document: &Html,
) -> Result<f32, Box<dyn error::Error + Send + Sync>> {
    let price_selector = Selector::parse(".desc_jeu .prix").unwrap();

    if let Some(price_element) = document.select(&price_selector).next() {
        let price = price_element.text().collect::<Vec<_>>().join("");

        return Ok(price.replace("€", ".").parse::<f32>()?);
    };

    Err("error get_okkazeo_price, no entry in select".into())
}

pub fn get_okkazeo_announce_name(
    document: &Html,
) -> Result<String, Box<dyn error::Error + Send + Sync>> {
    let name_selector = Selector::parse("div.large-12.cell h1").unwrap();

    if let Some(name_element) = document.select(&name_selector).next() {
        return Ok(name_element.text().collect::<Vec<_>>().join(""));
    };

    Err("error get_okkazeo_name, no entry in select".into())
}

pub fn get_okkazeo_announce_extension(
    document: &Html,
) -> Result<String, Box<dyn error::Error + Send + Sync>> {
    let extension_selector = Selector::parse("div.large-12.cell b").unwrap();

    if let Some(extension_element) = document.select(&extension_selector).next() {
        return Ok(extension_element.text().collect::<Vec<_>>().join(""));
    };

    Err("error get_okkazeo_extension, no entry in select".into())
}

pub fn get_okkazeo_announce_modification_date(
    document: &Html,
) -> Result<DateTime<Utc>, Box<dyn error::Error + Send + Sync>> {
    let re = Regex::new(r"Modifiée le (\d{2}/\d{2}/\d{2})").unwrap();

    if let Some(captures) = re.captures(&document.html()) {
        if let Some(date) = captures.get(1) {
            let naive_date: NaiveDate = NaiveDate::parse_from_str(date.as_str(), "%d/%m/%y")?;

            // this is a trick as okkazeo announces dont have time, only date, so in order to try to have
            // them with the same order as the website, I add the current time (reversed as we are going through pages
            // in reverse time order)
            let duration = NaiveTime::from_hms_opt(23, 59, 59)
                .unwrap()
                .signed_duration_since(Local::now().naive_utc().time());

            let naive_datetime: NaiveDateTime = naive_date.and_time(
                NaiveTime::from_hms_opt(
                    duration.num_hours() as u32,
                    duration.num_minutes() as u32 % 60,
                    duration.num_seconds() as u32 % 60,
                )
                .unwrap(),
            );
            let datetime_utc = DateTime::<Utc>::from_utc(naive_datetime, Utc);
            return Ok(datetime_utc);
        }
    }

    Err("error get_okkazeo_modification_date, no entry in select".into())
}

pub fn get_okkazeo_announce_image(
    document: &Html,
) -> Result<String, Box<dyn error::Error + Send + Sync>> {
    let image_selector = Selector::parse("div.image-wrapper.image img").unwrap();

    if let Some(image_element) = document.select(&image_selector).next() {
        return Ok(image_element.value().attr("src").unwrap().to_string());
    };

    Err("error get_okkazeo_image, no entry in select".into())
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

pub async fn download_okkazeo_game_image(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    log::debug!("[TASK] getting image from {}", url);
    let response = get(url).await?;
    let image_bytes = response.bytes().await?;

    let image_reader = ImageReader::new(std::io::Cursor::new(image_bytes))
        .with_guessed_format()
        .expect("Failed to guess image format");

    let image = image_reader.decode().unwrap();

    let mut bytes: Vec<u8> = Vec::new();
    image.write_to(
        &mut Cursor::new(&mut bytes),
        image::ImageOutputFormat::Jpeg(60),
    )?;

    let re = Regex::new(r#"/([^/]+)\.(jpg|png)$"#).unwrap();
    let mut name: &str = "unknown";
    if let Some(captures) = re.captures(url) {
        if let Some(filename) = captures.get(1) {
            name = filename.as_str();
        }
    }

    if !std::path::Path::new("img").exists() {
        std::fs::create_dir("img").unwrap();
    }
    let output_path = Path::new("img").join(format!("{}{}", name, ".jpg"));
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

pub async fn get_games_from_page(
    page: u32,
) -> Result<Vec<u32>, Box<dyn error::Error + Send + Sync>> {
    let search = format!("https:///www.okkazeo.com/jeux/arrivages?page={}", page);
    log::debug!("getting okkazeo page : {}", &search);

    let client = ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let response = client.get(search).send().await?;
    let content = response.bytes().await?;
    let document = Html::parse_document(std::str::from_utf8(&content)?);

    let mbs_selector = Selector::parse(".mbs").unwrap();
    let href_selector = Selector::parse(".mbs .h4-like.titre a[href]").unwrap();

    let mut links = Vec::new();
    for mbs_element in document.select(&mbs_selector) {
        for element in mbs_element.select(&href_selector) {
            if let Some(href) = element.value().attr("href") {
                let ids = href
                    .split('/')
                    .collect::<Vec<&str>>()
                    .last()
                    .unwrap()
                    .parse::<u32>()?;
                links.push(ids);
            }
        }
    }

    Ok(links)
}
