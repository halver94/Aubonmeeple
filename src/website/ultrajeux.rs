use regex::Regex;

use crate::website::helper::clean_name;

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
            ULTRAJEUX_STAT.with_label_values(&["success"]).inc();
            return Some((number, search));
        }
    }
    ULTRAJEUX_STAT.with_label_values(&["fail"]).inc();
    None
}

pub async fn get_ultrajeux_price_and_url_by_name(name: &str) -> Option<(f32, String)> {
    let search = format!(
        "https://www.ultrajeux.com/search3.php?text={}&submit=Ok",
        clean_name(name)
    );
    log::debug!("[TASK] search on ultrajeux by name: {}", &search);
    let content = reqwest::get(&search).await.unwrap().bytes().await.unwrap();
    let re = Regex::new(r#"produit_prix.*?class="prix.*?([\d,]+) "#).unwrap();
    let content_str: &str = &String::from_utf8_lossy(&content);
    for capture in re.captures_iter(content_str) {
        if let Some(value) = capture.get(1) {
            let number: f32 = value.as_str().replace(',', ".").parse().unwrap();
            ULTRAJEUX_STAT.with_label_values(&["success"]).inc();
            return Some((number, search));
        }
    }
    ULTRAJEUX_STAT.with_label_values(&["fail"]).inc();
    None
}

pub async fn get_ultrajeux_price_and_url(
    name: &str,
    barcode: Option<u64>,
) -> Option<(f32, String)> {
    if barcode.is_some() {
        if let Some((a, b)) = get_ultrajeux_price_and_url_by_barcode(barcode.unwrap()).await {
            return Some((a, b));
        }
    }
    get_ultrajeux_price_and_url_by_name(name).await
}

use lazy_static::lazy_static;
use prometheus::{register_int_counter_vec, IntCounterVec};
lazy_static! {
    static ref ULTRAJEUX_STAT: IntCounterVec = register_int_counter_vec!(
        "ultrajeux_stat",
        "Stat about parsing/fetch success/fail for this website",
        &["result"]
    )
    .unwrap();
}
