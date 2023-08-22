use serde::{Deserialize, Serialize};

use crate::game::Game;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    pub sort: String,
}

impl Default for Sort {
    fn default() -> Self {
        Self {
            sort: String::from("updated"),
        }
    }
}

impl Sort {
    pub fn sort(&self, games: Vec<Box<Game>>) -> Vec<Box<Game>> {
        let mut games = games;
        match self.sort.as_str() {
            "updated" => games.sort_by(|a, b| {
                b.okkazeo_announce
                    .last_modification_date
                    .cmp(&a.okkazeo_announce.last_modification_date)
            }),
            "note" => games.sort_by(|a, b| {
                b.review
                    .average_note
                    .partial_cmp(&a.review.average_note)
                    .unwrap()
            }),
            "deal" => {
                games.sort_by(|a, b| a.deal.deal_price.partial_cmp(&b.deal.deal_price).unwrap())
            }
            _ => {}
        };
        games
    }
}
