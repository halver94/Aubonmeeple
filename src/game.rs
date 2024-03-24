use crate::website::agorajeux::get_agorajeux_price_and_url_by_name;
use crate::website::knapix::get_knapix_prices;
use crate::website::ludifolie::get_ludifolie_price_and_url_by_name;
use crate::website::ludocortex::get_ludocortex_price_and_url;
use crate::website::okkazeo::{
    get_okkazeo_announce_page, get_okkazeo_barcode, get_okkazeo_city, get_okkazeo_seller,
};
use crate::website::philibert::get_philibert_price_and_url;
use chrono::{DateTime, Utc};
use feed_rs::model::Entry;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::error;

use crate::website::okkazeo::{
    download_okkazeo_game_image, get_okkazeo_announce_extension, get_okkazeo_announce_image,
    get_okkazeo_announce_modification_date, get_okkazeo_announce_name, get_okkazeo_announce_price,
    get_okkazeo_shipping, okkazeo_is_pro_seller,
};

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

pub async fn get_game_infos(
    entry: Option<&Entry>,
    id: u32,
) -> Result<Box<Game>, Box<dyn error::Error + Send + Sync>> {
    log::debug!("fetching game infos for id {:?}", id);

    let image_url: String;
    let mut game = Box::new(Game {
        okkazeo_announce: OkkazeoAnnounce {
            id,
            ..Default::default()
        },
        references: HashMap::<String, Reference>::new(),
        ..Default::default()
    });

    {
        let (document, _) = get_okkazeo_announce_page(id).await;
        game.okkazeo_announce.url = format!("https://www.okkazeo.com/annonces/view/{}", id);
        game.okkazeo_announce.price = get_okkazeo_announce_price(&document)?;
        game.okkazeo_announce.extension = get_okkazeo_announce_extension(&document)?;
        game.okkazeo_announce.last_modification_date = if let Some(e) = entry {
            e.updated.unwrap()
        } else {
            get_okkazeo_announce_modification_date(&document)?
        };
        image_url = get_okkazeo_announce_image(&document)?;
        game.okkazeo_announce.barcode = get_okkazeo_barcode(&document);
        game.okkazeo_announce.city = get_okkazeo_city(&document);
        game.okkazeo_announce.shipping = get_okkazeo_shipping(&document);
        if let Some(s) = get_okkazeo_seller(&document) {
            game.okkazeo_announce.seller = s;
        }
        game.okkazeo_announce.seller.is_pro = okkazeo_is_pro_seller(&document);

        let name = get_okkazeo_announce_name(&document)?;
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
    }
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

    /*if game.references.get("ultrajeux").is_none() {
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
