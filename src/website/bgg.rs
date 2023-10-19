use scraper::{Html, Selector};

use crate::{game::Reviewer, website::helper::{are_names_similar, clean_name}};

pub async fn get_bgg_note(name: &str) -> Option<Reviewer> {
    let name = clean_name(name);
    let search = format!(
        "https://boardgamegeek.com/geeksearch.php?action=search&objecttype=boardgame&q={}", name);
    log::debug!("[TASK] getting bgg note: {}\n", &search);
    let content = reqwest::get(&search).await.unwrap().bytes().await.unwrap();
    parse_bgg_document(&name, search, std::str::from_utf8(&content).unwrap())
}

fn parse_bgg_document(name: &str, search: String, doc : &str) -> Option<Reviewer> {
    let document = Html::parse_document(doc);

    let primary_selector = Selector::parse("a.primary").unwrap();

    // Sélecteur pour les éléments avec la classe 'collection_bggrating'
    let bggrating_selector = Selector::parse("td.collection_bggrating").unwrap();

    let selected_name = if let Some(primary) = document.select(&primary_selector).next() {
        primary.text().collect::<Vec<_>>().join("")
    } else {
        return None;
    };
    log::trace!("selected_name: {} vs name {}", selected_name, name);

    let mut bggrating_values = Vec::new();
    for bggrating in document.select(&bggrating_selector).skip(1).take(2) {
        let bggrating_value = bggrating.text().collect::<Vec<_>>().join("");
        let ratings = bggrating_value.trim();

        bggrating_values.push(ratings.to_string());
    }
    log::trace!("bggrating_values: {:#?}", bggrating_values);

    if bggrating_values.len() == 2 && are_names_similar(name, &selected_name) {
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

#[cfg(test)]
mod tests {
    use std::{fs, env};
    use log::Level;

    use crate::website::{bgg::parse_bgg_document, helper::clean_name};


    struct Test {
        name: String,
        note: f32,
        review_cnt: u32,
        document: String,
    }

    #[test]
    fn it_works() {
    env::set_var("RUST_LOG", "boardgame_finder=trace");
        let _ = env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(Level::Info.as_str()),
    )
    .try_init();

        log::trace!("starting bgg tests");
        let tests = vec![
            Test{ name: String::from("Lucky Bastard"), note: 5.0, review_cnt: 1, document: String::from("tests/bgg/test1.html")},
            Test{ name: String::from("Cartaventura : Versailles"), note: 6.79, review_cnt: 8, document: String::from("tests/bgg/test2.html")},
            Test{ name: String::from("Michel Strogoff VF"), note: 6.72, review_cnt: 621, document: String::from("tests/bgg/test3.html")},
            Test{ name: String::from("Tiny Epic Western Base"), note: 6.64, review_cnt: 4179, document: String::from("tests/bgg/test4.html")},
            Test{ name: String::from("Strife: Shadows & Steam"), note: 6.53, review_cnt: 138, document: String::from("tests/bgg/test5.html")},
            Test{ name: String::from("Runebound"), note: 6.22, review_cnt: 1577, document: String::from("tests/bgg/test6.html")},
        ];
        for test in tests.into_iter() {
            let name = clean_name(test.name.as_str());
            let html_doc = fs::read_to_string(test.document).expect("Should have been able to read the file");
            let review = parse_bgg_document(&name, String::new(), &html_doc).unwrap();
            assert_eq!(review.note, test.note);
            assert_eq!(review.number, test.review_cnt);
        }
    }
}
