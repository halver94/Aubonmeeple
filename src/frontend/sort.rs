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
    pub fn sort(&self, games: Vec<Game>) -> Vec<Game> {
        //println!("filters : {:#?}", self);

        let mut games = games;
        match self.sort.as_str() {
            "updated" => games.sort_by(|a, b| {
                b.okkazeo_announce
                    .last_modification_date
                    .cmp(&a.okkazeo_announce.last_modification_date)
            }),
            "bgg" => games.sort_by(|a, b| b.note_bgg.partial_cmp(&a.note_bgg).unwrap()),
            "trictrac" => {
                games.sort_by(|a, b| b.note_trictrac.partial_cmp(&a.note_trictrac).unwrap())
            }
            "deal" => {
                games.sort_by(|a, b| a.deal.deal_price.partial_cmp(&b.deal.deal_price).unwrap())
            }
            _ => {}
        };
        games
    }
}
