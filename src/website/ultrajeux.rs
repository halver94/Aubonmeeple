use log::debug;
use regex::Regex;

pub async fn get_ultrajeux_price_and_url_by_barcode(barcode: u64) -> Option<(f32, String)> {
    let search = format!(
        "https://www.ultrajeux.com/search3.php?text={}&submit=Ok",
        barcode
    );
    log::debug!("[TASK] search on ultrajeux by barcode: {}", &search);
    let content = reqwest::get(&search).await.unwrap().bytes().await.unwrap();
    let re = Regex::new(r#"produit_prix.*?class="prix.*?([\d,]+) "#).unwrap();
    let content_str: &str = &String::from_utf8_lossy(&content);
    for capture in re.captures_iter(content_str) {
        if let Some(value) = capture.get(1) {
            let number: f32 = value.as_str().replace(',', ".").parse().unwrap();
            return Some((number, search));
        }
    }
    None
}

pub async fn get_ultrajeux_price_and_url_by_name(name: &String) -> Option<(f32, String)> {
    let search = format!(
        "https://www.ultrajeux.com/search3.php?text={}&submit=Ok",
        name
    );
    log::debug!("[TASK] search on ultrajeux by name: {}", &search);
    let content = reqwest::get(&search).await.unwrap().bytes().await.unwrap();
    let re = Regex::new(r#"produit_prix.*?class="prix.*?([\d,]+) "#).unwrap();
    let content_str: &str = &String::from_utf8_lossy(&content);
    for capture in re.captures_iter(content_str) {
        if let Some(value) = capture.get(1) {
            let number: f32 = value.as_str().replace(',', ".").parse().unwrap();
            return Some((number, search));
        }
    }
    None
}

pub async fn get_ultrajeux_price_and_url(
    name: &String,
    barcode: Option<u64>,
) -> Option<(f32, String)> {
    if barcode.is_some() {
        if let Some((a, b)) = get_ultrajeux_price_and_url_by_barcode(barcode.unwrap()).await {
            return Some((a, b));
        }
    }
    get_ultrajeux_price_and_url_by_name(name).await
}
