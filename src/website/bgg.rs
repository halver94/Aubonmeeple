use scraper::{Html, Selector};

use crate::game::Reviewer;

pub async fn get_bgg_note(name: &str) -> Option<Reviewer> {
    let search = format!(
        "https://boardgamegeek.com/geeksearch.php?action=search&objecttype=boardgame&q={}",
        name.replace(' ', "-")
            .replace([':', '\''], "")
            .to_lowercase()
    );
    log::debug!("[TASK] getting bgg note: {}\n", &search);
    let content = reqwest::get(&search).await.unwrap().bytes().await.unwrap();
    let document = Html::parse_document(std::str::from_utf8(&content).unwrap());

    let primary_selector = Selector::parse("a.primary").unwrap();

    // Sélecteur pour les éléments avec la classe 'collection_bggrating'
    let bggrating_selector = Selector::parse("td.collection_bggrating").unwrap();

    let selected_name = if let Some(primary) = document.select(&primary_selector).next() {
        primary.text().collect::<Vec<_>>().join("")
    } else {
        return None;
    };

    let mut bggrating_values = Vec::new();
    for bggrating in document.select(&bggrating_selector).skip(1).take(2) {
        let bggrating_value = bggrating.text().collect::<Vec<_>>().join("");
        let ratings = bggrating_value.trim();

        bggrating_values.push(ratings.to_string());
    }

    if bggrating_values.len() == 2 && name.to_lowercase() == selected_name.to_lowercase() {
        let rating = bggrating_values[0].clone().parse::<f32>().unwrap_or(0.0);
        let review_cnt = bggrating_values[1].clone().parse::<u32>().unwrap_or(0);
        return Some(Reviewer {
            name: "bgg".to_string(),
            note: rating,
            number: review_cnt,
            url: search,
        });
    }

    None
}
