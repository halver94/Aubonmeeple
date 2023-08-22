use scraper::{Html, Selector};

use crate::game::Reviewer;

pub async fn get_trictrac_note(name: &str) -> Option<Reviewer> {
    let search = format!(
        "https://www.trictrac.net/recherche?search={}",
        name.replace(' ', "-")
            .replace([':', '\''], "")
            .to_lowercase()
    );
    log::debug!("[TASK] getting tric trac note: {}\n", &search);
    let content = reqwest::get(&search).await.unwrap().bytes().await.unwrap();
    let document = Html::parse_document(std::str::from_utf8(&content).unwrap());

    let item_selector = Selector::parse("div.item").unwrap();

    for item in document.select(&item_selector) {
        let title_selector = Selector::parse("span[itemprop=name]").unwrap();
        let title = item
            .select(&title_selector)
            .next()
            .map(|node| node.inner_html());

        let rating_value_selector = Selector::parse("[itemprop=ratingValue]").unwrap();
        let rating_value = item
            .select(&rating_value_selector)
            .next()
            .and_then(|node| node.value().attr("content"))
            .and_then(|content| content.parse::<f32>().ok());

        let review_count_selector = Selector::parse("[itemprop=reviewCount]").unwrap();
        let review_count = item
            .select(&review_count_selector)
            .next()
            .and_then(|node| node.value().attr("content"))
            .and_then(|content| content.parse::<u32>().ok());

        log::debug!(
            "[TASK] title: {:#?}, review value {:#?}, count : {:#?}",
            title,
            rating_value,
            review_count
        );

        if rating_value.is_none() || title.is_none() || review_count.is_none() {
            return None;
        }

        if title.unwrap().to_lowercase() == name.to_lowercase() {
            return Some(Reviewer {
                name: "trictrac".to_string(),
                note: rating_value.unwrap(),
                number: review_count.unwrap(),
                url: search,
            });
        }
    }

    None
}
