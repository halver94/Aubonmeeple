use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap};

use crate::website::{bgg::get_bgg_note, trictrac::get_trictrac_note};

#[derive(Debug, Default, Clone)]
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

#[derive(Debug, Default, Clone)]
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

#[derive(Debug, Default, Clone)]
pub struct Game {
    pub okkazeo_announce: OkkazeoAnnounce,
    pub references: HashMap<String, Reference>,
    pub review: Review,
    pub deal: Deal,
}

#[derive(Debug, Default, Clone)]
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
            log::debug!("[TASK] no references for {}", self.okkazeo_announce.name);
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
        let reviewer = get_trictrac_note(&self.okkazeo_announce.name).await;
        if reviewer.is_some() {
            self.review
                .reviews
                .insert("trictrac".to_string(), reviewer.unwrap());
        } else {
            log::debug!(
                "[TASK] cannot get trictrac note for {}",
                self.okkazeo_announce.name
            );
        }

        let reviewer = get_bgg_note(&self.okkazeo_announce.name).await;
        if reviewer.is_some() {
            self.review
                .reviews
                .insert("bgg".to_string(), reviewer.unwrap());
        } else {
            log::debug!(
                "[TASK] cannot get bgg note for {}",
                self.okkazeo_announce.name
            );
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
