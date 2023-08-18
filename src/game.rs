use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Games {
    pub games: Vec<Game>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub name: String,
    pub price: f32,
    pub url: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Seller {
    pub name: String,
    pub url: String,
    pub nb_announces: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OkkazeoAnnounce {
    pub id: u32,
    pub name: String,
    pub price: f32,
    pub url: String,
    pub extension: String,
    pub seller: Seller,
    pub barcode: Option<u64>,
    pub city: Option<String>,
    pub last_modification_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Game {
    pub okkazeo_announce: OkkazeoAnnounce,
    pub references: HashMap<String, Reference>,
    pub note_trictrac: f32,
    pub review_count_trictrac: u32,
    pub note_bgg: f32,
    pub review_count_bgg: u32,
}

impl Ord for Game {
    fn cmp(&self, other: &Self) -> Ordering {
        self.okkazeo_announce
            .last_modification_date
            .cmp(&other.okkazeo_announce.last_modification_date)
    }
}
impl PartialOrd for Game {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
    fn get_deal_advantage(&self) -> Option<(i32, i32)> {
        // okkazeo is counted as a ref, so we need at least 2 refs
        if self.references.is_empty() {
            return None;
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
            return None;
        }
        Some((economy, percent))
    }
}

impl Games {
    pub fn new() -> Games {
        Games {
            games: Vec::<Game>::new(),
        }
    }

    pub fn create_html_table(self) -> String {
        let mut table = String::new();
        table.push_str(
            "<style>
                    table {
                        border-collapse: collapse;
                        width: 100%;
                    }
                    th, td {
                        border: 1px solid black;
                        padding: 8px;
                        text-align: center;
                    }
                    th {
                        background-color: lightgray;
                    }
                    </style>",
        );
        table.push_str("<table>");
        table.push_str("<tr><th>Updated</th><th>Name</th><th>City</th><th>Seller</th><th>Deal</th><th>Okkazeo</th><th>Philibert</th><th>Agorajeux</th><th>Ultrajeux</th><th>Ludocortex</th><th>TricTrac Note</th><th>BGG Note</th></tr>");

        for game in self.games.iter().rev() {
            table.push_str("<tr>");
            table.push_str(&format!(
                "<td>{}</td>",
                game.okkazeo_announce
                    .last_modification_date
                    .unwrap()
                    .format("%d/%m/%Y %H:%M")
            ));
            table.push_str(&format!(
                "<td>{}<br>({})</td>",
                game.okkazeo_announce.name, game.okkazeo_announce.extension
            ));
            table.push_str(&format!(
                "<td>{}</td>",
                game.okkazeo_announce.city.clone().unwrap_or(String::new())
            ));
            table.push_str(&format!(
                "<td><a href=\"{}\">{} <br>({} announces)</a></td>",
                game.okkazeo_announce.seller.url,
                game.okkazeo_announce.seller.name,
                game.okkazeo_announce.seller.nb_announces
            ));

            if let Some((diff_price, percent_saved)) = game.get_deal_advantage() {
                table.push_str(&format!(
                    "<td style=\"color: {}\">{}{}€ ({}{}%)</td>",
                    if diff_price < 0 { "green" } else { "red" },
                    if diff_price >= 0 { "+" } else { "" },
                    diff_price,
                    if percent_saved > 0 { "+" } else { "" },
                    percent_saved,
                ));
            } else {
                table.push_str("<td>-</td>");
            }

            table.push_str(&format!(
                "<td><a href=\"{}\">{} &euro;</a></td>",
                game.okkazeo_announce.url, game.okkazeo_announce.price,
            ));

            if game.references.get("philibert").is_some() {
                table.push_str(&format!(
                    "<td><a href=\"{}\">{} &euro;</a></td>",
                    game.references.get("philibert").unwrap().url,
                    game.references.get("philibert").unwrap().price,
                ));
            } else {
                table.push_str("<td>-</td>");
            }
            if game.references.get("agorajeux").is_some() {
                table.push_str(&format!(
                    "<td><a href=\"{}\">{} &euro;</a></td>",
                    game.references.get("agorajeux").unwrap().url,
                    game.references.get("agorajeux").unwrap().price,
                ));
            } else {
                table.push_str("<td>-</td>");
            }
            if game.references.get("ultrajeux").is_some() {
                table.push_str(&format!(
                    "<td><a href=\"{}\">{} &euro;</a></td>",
                    game.references.get("ultrajeux").unwrap().url,
                    game.references.get("ultrajeux").unwrap().price,
                ));
            } else {
                table.push_str("<td>-</td>");
            }
            if game.references.get("ludocortex").is_some() {
                table.push_str(&format!(
                    "<td><a href=\"{}\">{} &euro;</a></td>",
                    game.references.get("ludocortex").unwrap().url,
                    game.references.get("ludocortex").unwrap().price,
                ));
            } else {
                table.push_str("<td>-</td>");
            }
            table.push_str(&format!(
                "<td>{} ({} reviews)</td>",
                game.note_trictrac, game.review_count_trictrac
            ));
            table.push_str(&format!(
                "<td>{} ({} reviews)</td>",
                game.note_bgg, game.review_count_bgg
            ));
            table.push_str("</tr>");
        }

        table.push_str("</table>");
        table
    }
}
