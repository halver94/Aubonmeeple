use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap};

use crate::{
    frontend::server::State,
    website::{bgg::get_bgg_note, trictrac::get_trictrac_note},
};

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
pub struct Shipping {
    pub handshake: bool,
    pub ships: HashMap<String, f32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Seller {
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
    pub shipping: Shipping,
    pub seller: Seller,
    pub barcode: Option<u64>,
    pub city: Option<String>,
    pub last_modification_date: Option<DateTime<Utc>>,
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
        log::debug!(
            "[TASK] calculating deal advantage for {}",
            self.okkazeo_announce.name
        );
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
        log::debug!("[TASK] getting reviews for {}", self.okkazeo_announce.name);
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

        let mut total_reviewer = 0;
        let mut note = 0.0;
        for val in self.review.reviews.values() {
            total_reviewer += val.number;
            note += val.note * val.number as f32
        }

        if total_reviewer == 0 || note == 0.0 {
            self.review.average_note = 0.0;
        } else {
            self.review.average_note = note / total_reviewer as f32;
        }
    }
}

impl Games {
    pub fn new() -> Games {
        Games {
            games: Vec::<Box<Game>>::new(),
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
                "{}{}{}{}{}{}{}",
                r#"<tr>
            <th>Updated <button onclick="window.location.href='/?"#,
                params,
                r#"&sort=updated';">Sort</button></th>
            <th>Name</th>
            <th>City</th>
            <th>Seller</th>
            <th>Shipping</th>
            <th>Deal <button onclick="window.location.href='/?"#,
                params,
                r#"&sort=deal';">Sort</button></th>
            <th>Okkazeo</th>
            <th>Shops</th>
            <th>Note <button onclick="window.location.href='/?"#,
                params,
                r#"&sort=note';">Sort</button></th>
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
                "<td><a href=\"{}\">{} {}<br>({} announces)</a></td>",
                game.okkazeo_announce.seller.url,
                game.okkazeo_announce.seller.name,
                if game.okkazeo_announce.seller.is_pro {
                    "- PRO"
                } else {
                    ""
                },
                game.okkazeo_announce.seller.nb_announces
            ));

            table.push_str("<td>");
            if game.okkazeo_announce.shipping.handshake {
                table.push_str("- Hand delivery <br>");
            }

            for (key, val) in game.okkazeo_announce.shipping.ships.iter() {
                table.push_str(&format!("- {} : {}€<br>", key, val));
            }

            table.push_str("</td>");

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

            table.push_str("<td>");
            if game.references.is_empty() {
                table.push_str("-");
            } else {
                for val in game.references.values() {
                    table.push_str(&format!(
                        "<a href=\"{}\">{} : {} &euro;</a><br>",
                        val.url, val.name, val.price,
                    ));
                }
            }
            table.push_str("</td>");

            if game.review.average_note == 0.0 {
                table.push_str("<td>-</td>");
            } else {
                table.push_str(&format!(
                    "<td>Average note : {:.2}<br><br>",
                    game.review.average_note,
                ));

                for val in game.review.reviews.values() {
                    table.push_str(&format!(
                        "<a href=\"{}\">{}: {} ({} reviews)</a><br>",
                        val.url, val.name, val.note, val.number
                    ));
                }
                table.push_str("</td>");
            }
            table.push_str("</tr>");
        }

        table.push_str("</table>");
        table
    }
}
