use std::error;

use scraper::{Html, Selector};

use crate::{game::{Game, Reference}, website::helper::clean_name};

pub async fn get_knapix_prices(game: &mut Game) -> Result<(), Box<dyn error::Error>> {
    let name = clean_name(&game.okkazeo_announce.name).replace(' ', "+");
    let search = format!(
        "https://www.knapix.com/comparateur.php?nom_jeu={}&checkbox-exact=on&affiner=",
        name
    );

    log::debug!("[TASK] searching knapix {}", search);
    let content = reqwest::get(search).await?.bytes().await?;
    let document = Html::parse_document(std::str::from_utf8(&content)?);

    // choper <tr data-href="/r/127347999"> pou rla redirection vers le site
    let row_selector = Selector::parse("tr[data-href]")?;
    let img_selector = Selector::parse("img[alt]")?;
    let price_selector = Selector::parse(".prix")?;

    for row in document.select(&row_selector) {
        let url = format!(
            "{}{}",
            "https://www.knapix.com",
            row.value().attr("data-href").unwrap_or_default()
        );
        let img_element = row.select(&img_selector).next();
        if let Some(img) = img_element {
            let alt_value = img.value().attr("alt").unwrap_or_default().to_lowercase();

            let price_element = row.select(&price_selector).next();
            if let Some(price) = price_element {
                let price_text = price.text().collect::<String>();
                let price = price_text
                    .trim()
                    .replace(" €", "")
                    .replace(',', ".")
                    .parse::<f32>()?;
                match alt_value.as_str() {
                    "agorajeux" | "philibert" | "ultrajeux" => {
                        game.references.insert(
                            String::from(alt_value.as_str()),
                            Reference {
                                name: String::from(alt_value.as_str()),
                                price: price,
                                url,
                            },
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
