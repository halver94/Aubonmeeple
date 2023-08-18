use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap};

use crate::frontend::server::State;

#[derive(Debug, Default, Clone)]
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

#[derive(Debug, Default, Clone)]
pub struct OkkazeoAnnounce {
    pub id: u32,
    pub name: String,
    pub image: String,
    pub price: f32,
    pub url: String,
    pub extension: String,
    pub seller: Seller,
    pub barcode: Option<u64>,
    pub city: Option<String>,
    pub last_modification_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Default, Clone)]
pub struct Game {
    pub okkazeo_announce: OkkazeoAnnounce,
    pub references: HashMap<String, Reference>,
    pub deal: Deal,
    pub note_trictrac: f32,
    pub review_count_trictrac: u32,
    pub note_bgg: f32,
    pub review_count_bgg: u32,
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
}

impl Games {
    pub fn new() -> Games {
        Games {
            games: Vec::<Game>::new(),
        }
    }

    pub fn create_html_table(&self, state: &State) -> String {
        let params = format!(
            "page={}&per_page={}{}{}",
            state.pagination.page,
            state.pagination.per_page,
            if state.filters.city.is_some() {
                format!("&city={}", state.filters.city.as_ref().unwrap())
            } else {
                String::new()
            },
            if state.filters.name.is_some() {
                format!("&name={}", state.filters.name.as_ref().unwrap())
            } else {
                String::new()
            },
        );

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
                   
                    .flex-container {
                        display: flex;
                        align-items: center; /* Alignement vertical */
                    }
                    .flex-container img {
                        margin-right: 10px; /* Espacement entre l'image et le texte */
                    }

                    </style>",
        );
        table.push_str("<table>");
        table.push_str(
            format!(
                "{}{}{}{}{}{}{}{}{}",
                r#"<tr>
            <th>Updated <button onclick="window.location.href='/?"#,
                params,
                r#"&sort=updated';">Sort</button></th>
            <th>Name</th>
            <th>City</th>
            <th>Seller</th>
            <th>Deal <button onclick="window.location.href='/?"#,
                params,
                r#"&sort=deal';">Sort</button></th>
            <th>Okkazeo</th>
            <th>Philibert</th>
            <th>Agorajeux</th>
            <th>Ultrajeux</th>
            <th>Ludocortex</th>
            <th>TricTrac Note <button onclick="window.location.href='/?"#,
                params,
                r#"&sort=trictrac';">Sort</button></th>
            <th>BGG Note <button onclick="window.location.href='/?"#,
                params,
                r#"&sort=bgg';">Sort</button></th>
        </tr>"#
            )
            .as_str(),
        );

        for game in self.games.iter() {
            table.push_str("<tr>");
            table.push_str(&format!(
                "<td>{}</td>",
                game.okkazeo_announce
                    .last_modification_date
                    .unwrap()
                    .format("%d/%m/%Y %H:%M")
            ));
            table.push_str(&format!(
                "<td>
                    <div class=\"flex-container\">
                        <img src=\"{}\" alt=\"fail\" width=\"100\" height=\"100\" />
                        {}<br>({})
                    </div>
                </td>",
                game.okkazeo_announce.image,
                game.okkazeo_announce.name,
                game.okkazeo_announce.extension
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

            if game.deal.deal_price != 0 {
                table.push_str(&format!(
                    "<td style=\"color: {}\">{}{}€ ({}{}%)</td>",
                    if game.deal.deal_price < 0 {
                        "green"
                    } else {
                        "red"
                    },
                    if game.deal.deal_price >= 0 { "+" } else { "" },
                    game.deal.deal_price,
                    if game.deal.deal_percentage > 0 {
                        "+"
                    } else {
                        ""
                    },
                    game.deal.deal_percentage,
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
