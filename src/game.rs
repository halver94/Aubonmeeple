use crate::website::agorajeux::get_agorajeux_price_and_url_by_name;
use crate::website::knapix::get_knapix_prices;
use crate::website::ludifolie::get_ludifolie_price_and_url_by_name;
use crate::website::ludocortex::get_ludocortex_price_and_url;
use crate::website::philibert::get_philibert_price_and_url;
use crate::website::ultrajeux::get_ultrajeux_price_and_url;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::error;

use crate::website::okkazeo::{download_okkazeo_game_image, Row};

use crate::website::bgg::get_bgg_note;

#[derive(Debug, Default, Clone, Serialize)]
pub struct Games {
    pub games: Vec<Box<Game>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub name: String,
    pub price: f32,
    pub url: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Review {
    pub reviews: HashMap<String, Reviewer>,
    pub average_note: f32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Reviewer {
    pub name: String,
    pub url: String,
    pub note: f32,
    pub number: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Seller {
    pub id: u32,
    pub name: String,
    pub url: String,
    pub nb_announces: u32,
    pub is_pro: bool,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct OkkazeoAnnounce {
    pub id: u32,
    pub name: String,
    pub image: String,
    pub price: f32,
    pub url: String,
    pub extension: String,
    pub shipping: HashMap<String, f32>,
    pub seller: Seller,
    pub barcode: Option<u64>,
    pub city: Option<String>,
    pub last_modification_date: DateTime<Utc>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Game {
    pub okkazeo_announce: OkkazeoAnnounce,
    pub references: HashMap<String, Reference>,
    pub review: Review,
    pub deal: Deal,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Deal {
    pub deal_price: i32,
    pub deal_percentage: i32,
}

impl Ord for Game {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .okkazeo_announce
            .last_modification_date
            .cmp(&self.okkazeo_announce.last_modification_date)
    }
}
impl PartialOrd for Game {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.cmp(self))
    }
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.okkazeo_announce.last_modification_date
            == other.okkazeo_announce.last_modification_date
    }
}

impl Eq for Game {}

impl Game {
    pub fn get_deal_advantage(&mut self) {
        // okkazeo is counted as a ref, so we need at least 2 refs
        if self.references.is_empty() {
            log::debug!("no references for {}", self.okkazeo_announce.name);
            return;
        }

        let mut min_price = 0.0;
        let mut first_ref = true;
        for (_, reference) in self.references.iter() {
            if first_ref {
                min_price = reference.price;
                first_ref = false;
            }

            if min_price > reference.price {
                min_price = reference.price;
            }
        }
        let percent: i32 = ((self.okkazeo_announce.price * 100.0) / min_price).round() as i32 - 100;
        let economy = (self.okkazeo_announce.price - min_price).round() as i32;

        if economy == 0 || percent == 0 {
            return;
        }
        self.deal.deal_price = economy;
        self.deal.deal_percentage = percent;
    }

    pub async fn get_reviews(&mut self) {
        match get_bgg_note(&self.okkazeo_announce.name).await {
            Err(e) => log::error!("error getting bgg note : {}", e),
            Ok(v) => {
                if let Some(r) = v {
                    self.review.reviews.insert("bgg".to_string(), r);
                } else {
                    log::debug!("cannot get bgg note for {}", self.okkazeo_announce.name);
                }
            }
        }
        self.review.compute_average_note();
    }

    pub fn update_game(&mut self, row: Row) {
        self.okkazeo_announce.price = row.prix_annonce;

        let naive_date: NaiveDateTime =
            match NaiveDateTime::parse_from_str(row.date.as_str(), "%Y-%m-%d %H:%M:%S") {
                Ok(dt) => dt,
                Err(e) => {
                    log::error!("Failed to parse datetime \"{}\" : {}", row.date, e);
                    return;
                }
            };
        let datetime_utc = DateTime::from_naive_utc_and_offset(naive_date, Utc);
        self.okkazeo_announce.last_modification_date = datetime_utc;

        self.get_deal_advantage();

        if let Some(v) = row.shop2shop {
            self.okkazeo_announce
                .shipping
                .insert("shop2shop".to_string(), v);
        }
        if let Some(v) = row.colissimo {
            self.okkazeo_announce
                .shipping
                .insert("colissimo".to_string(), v);
        }

        if let Some(v) = row.mondial_relay {
            self.okkazeo_announce
                .shipping
                .insert("mondial_relay".to_string(), v);
        }

        if let Some(v) = row.relais_colis {
            self.okkazeo_announce
                .shipping
                .insert("relais_colis".to_string(), v);
        }

        if row.rmp {
            self.okkazeo_announce
                .shipping
                .insert("hand_delivery".to_string(), 0.0);
        }
    }
}

impl Games {
    pub fn new() -> Games {
        Games {
            games: Vec::<Box<Game>>::new(),
        }
    }
}

impl Review {
    pub fn compute_average_note(&mut self) {
        let mut total_reviewer = 0;
        let mut note = 0.0;
        for val in self.reviews.values() {
            total_reviewer += val.number;
            note += val.note * val.number as f32
        }

        if total_reviewer == 0 || note == 0.0 {
            self.average_note = 0.0;
        } else {
            self.average_note = note / total_reviewer as f32;
        }
    }
}

pub async fn get_game_infos(row: Row) -> Result<Box<Game>, Box<dyn error::Error + Send + Sync>> {
    log::debug!("Getting game infos, parsing row");
    let mut game = Box::new(Game {
        okkazeo_announce: OkkazeoAnnounce {
            id: row.id,
            price: row.prix_annonce,
            url: row.url_announce,
            extension: row.kind,
            barcode: row.ean,
            shipping: HashMap::new(),
            city: Some(match row.zipcode {
                Some(z) => format!("{} ({})", row.city, z),
                None => format!("{}", row.city),
            }),
            ..Default::default()
        },
        references: HashMap::<String, Reference>::new(),
        ..Default::default()
    });

    let naive_date: NaiveDateTime =
        match NaiveDateTime::parse_from_str(row.date.as_str(), "%Y-%m-%d %H:%M:%S") {
            Ok(dt) => dt,
            Err(e) => {
                log::error!("Failed to parse datetime \"{}\" : {}", row.date, e);
                return Err(Box::new(e));
            }
        };
    let datetime_utc = DateTime::from_naive_utc_and_offset(naive_date, Utc);
    game.okkazeo_announce.last_modification_date = datetime_utc;

    let vendor_id = row
        .url_vendor
        .split('/')
        .last()
        .unwrap_or("")
        .parse::<u32>()
        .unwrap_or(0);

    game.okkazeo_announce.seller = Seller {
        id: vendor_id,
        name: row.vendor,
        url: row.url_vendor,
        nb_announces: 0,
        is_pro: row.pro,
    };

    if let Some(v) = row.shop2shop {
        game.okkazeo_announce
            .shipping
            .insert("shop2shop".to_string(), v);
    }
    if let Some(v) = row.colissimo {
        game.okkazeo_announce
            .shipping
            .insert("colissimo".to_string(), v);
    }

    if let Some(v) = row.mondial_relay {
        game.okkazeo_announce
            .shipping
            .insert("mondial_relay".to_string(), v);
    }

    if let Some(v) = row.relais_colis {
        game.okkazeo_announce
            .shipping
            .insert("relais_colis".to_string(), v);
    }

    if row.rmp {
        game.okkazeo_announce
            .shipping
            .insert("hand_delivery".to_string(), 0.0);
    }

    let name = row.name;
    let mut inside_parentheses = false;
    let mut name_result = String::new();

    for c in name.chars() {
        match c {
            '(' => inside_parentheses = true,
            ')' => inside_parentheses = false,
            _ if !inside_parentheses => name_result.push(c),
            _ => (),
        }
    }
    game.okkazeo_announce.name = name_result;

    let image_url = row.url_image;
    let image = download_okkazeo_game_image(&image_url).await?;
    game.okkazeo_announce.image = image;

    if let Err(e) = get_knapix_prices(&mut game).await {
        log::error!("Error gettin knapix prices : {:?}", e);
    }

    if game.references.get("philibert").is_none() {
        match get_philibert_price_and_url(
            &game.okkazeo_announce.name,
            game.okkazeo_announce.barcode,
        )
        .await
        {
            Err(e) => log::error!("error getting philibert price : {}", e),
            Ok(v) => {
                if let Some((price, url)) = v {
                    game.references.insert(
                        "philibert".to_string(),
                        Reference {
                            name: "philibert".to_string(),
                            price,
                            url,
                        },
                    );
                }
            }
        }
    }

    if game.references.get("agorajeux").is_none() {
        match get_agorajeux_price_and_url_by_name(&game.okkazeo_announce.name).await {
            Err(e) => log::error!("error getting agorajeux price : {}", e),
            Ok(v) => {
                if let Some((price, url)) = v {
                    game.references.insert(
                        "agorajeux".to_string(),
                        Reference {
                            name: "agorajeux".to_string(),
                            price,
                            url,
                        },
                    );
                }
            }
        }
    }

    if game.references.get("ludifolie").is_none() {
        match get_ludifolie_price_and_url_by_name(&game.okkazeo_announce.name).await {
            Err(e) => log::error!("error getting ludifolie price : {}", e),
            Ok(v) => {
                if let Some((price, url)) = v {
                    game.references.insert(
                        "ludifolie".to_string(),
                        Reference {
                            name: "ludifolie".to_string(),
                            price,
                            url,
                        },
                    );
                }
            }
        }
    }

    /*
    if game.references.get("ultrajeux").is_none() {
        match get_ultrajeux_price_and_url(
            &game.okkazeo_announce.name,
            game.okkazeo_announce.barcode,
        )
        .await
        {
            Err(e) => log::error!("error getting ultrajeux price : {}", e),
            Ok(v) => {
                if let Some((price, url)) = v {
                    game.references.insert(
                        "ultrajeux".to_string(),
                        Reference {
                            name: "ultrajeux".to_string(),
                            price,
                            url,
                        },
                    );
                }
            }
        }
    }*/

    if game.references.get("ludocortex").is_none() {
        match get_ludocortex_price_and_url(
            &game.okkazeo_announce.name,
            game.okkazeo_announce.barcode,
        )
        .await
        {
            Err(e) => log::error!("error getting ultrajeux price : {}", e),
            Ok(v) => {
                if let Some((price, url)) = v {
                    game.references.insert(
                        "ludocortex".to_string(),
                        Reference {
                            name: "ludocortex".to_string(),
                            price,
                            url,
                        },
                    );
                }
            }
        }
    }

    game.get_reviews().await;
    game.get_deal_advantage();

    log::debug!("returning game {:?}", game);
    Ok(game)
}
