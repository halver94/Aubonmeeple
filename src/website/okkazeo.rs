use std::{
    fs::File,
    io::{Cursor, Write},
    path::Path,
};

use image::io::Reader as ImageReader;
use regex::Regex;
use serde::Deserialize;

use crate::httpclient;

pub async fn download_okkazeo_game_image(
    url: &str,
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    log::debug!("getting image from {}", url);
    let response = httpclient::get(url).await?;
    let image_bytes = response.bytes().await?;

    let image_reader = ImageReader::new(std::io::Cursor::new(image_bytes)).with_guessed_format()?;

    let image = image_reader.decode()?;

    let mut bytes: Vec<u8> = Vec::new();
    image.write_to(
        &mut Cursor::new(&mut bytes),
        image::ImageOutputFormat::Jpeg(60),
    )?;

    let re = Regex::new(r"/([^/]+)\.(jpg|png)$").unwrap();
    let mut name: &str = "unknown";
    if let Some(captures) = re.captures(url) {
        if let Some(filename) = captures.get(1) {
            name = filename.as_str();
        }
    }

    if !std::path::Path::new("img").exists() {
        std::fs::create_dir("img")?;
    }
    let output_path = Path::new("img").join(format!("{}{}", name, ".jpg"));
    let mut output_file = File::create(&output_path)?;
    output_file.write_all(&bytes)?;

    Ok(output_path.to_str().unwrap().to_string())
}

// Id|URL annonce|Image|Titre|EAN|Type|Date|Prix|Prix public|Vendeur|Pro|Profil vendeur|Code postal|Ville|RMP|Colissimo|Mondial Relay|Relais Colis|Shop2Shop
#[derive(Debug, serde::Deserialize, PartialEq, Clone)]
pub struct Row {
    #[serde(rename = "Id")]
    pub id: u32,
    #[serde(rename = "URL annonce")]
    pub url_announce: String,
    #[serde(rename = "Image")]
    pub url_image: String,
    #[serde(rename = "Titre")]
    pub name: String,
    #[serde(rename = "EAN")]
    pub ean: Option<u64>,
    #[serde(rename = "Type")]
    pub kind: String,
    #[serde(rename = "Date")]
    pub date: String,
    #[serde(rename = "Prix")]
    pub prix_annonce: f32,
    #[serde(rename = "Prix public")]
    pub prix_min: f32,
    #[serde(rename = "Vendeur")]
    pub vendor: String,
    #[serde(deserialize_with = "deserialize_bool")]
    #[serde(rename = "Pro")]
    pub pro: bool,
    #[serde(rename = "Profil vendeur")]
    pub url_vendor: String,
    #[serde(rename = "Code postal")]
    pub zipcode: Option<u32>,
    #[serde(rename = "Ville")]
    pub city: String,
    #[serde(rename = "RMP")]
    #[serde(deserialize_with = "deserialize_bool")]
    pub rmp: bool,
    #[serde(rename = "Colissimo")]
    pub colissimo: Option<f32>,
    #[serde(rename = "Mondial Relay")]
    pub mondial_relay: Option<f32>,
    #[serde(rename = "Relais Colis")]
    pub relais_colis: Option<f32>,
    #[serde(rename = "Shop2Shop")]
    pub shop2shop: Option<f32>,
}

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    if let Some(v) = opt {
        if v == "1" {
            return Ok(true);
        }
    }
    Ok(false)
}

pub async fn get_okkazeo_csv(url: String) -> Result<Vec<Row>, anyhow::Error> {
    log::debug!("getting csv file");
    let content = httpclient::get(url).await?.bytes().await?;
    let cursor = Cursor::new(content);
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'|')
        .from_reader(cursor);
    Ok(reader
        .deserialize::<Row>()
        .filter_map(|e| match e {
            Ok(r) => Some(r),
            Err(er) => {
                log::error!("error deserializing : {}", er);
                None
            }
        })
        .collect::<Vec<Row>>())
}
