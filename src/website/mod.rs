use async_trait::async_trait;
use scraper::Html;

pub mod agorajeux;
pub mod bgg;
pub mod helper;
pub mod knapix;
pub mod ludifolie;
pub mod ludocortex;
pub mod okkazeo;
pub mod philibert;
pub mod ultrajeux;

pub enum Reseller {
    Philibert,
    Agorajeux,
    Espritjeu,
    Ludifolie,
    Ludocortex,
    Ultrajeux,
}
pub struct StandardReseller<Reseller> {
    pub reseller: Reseller,
}

#[async_trait]
pub trait StandardResellerTrait {
    async fn get_price_and_url_by_name();
    async fn get_price_and_url_by_barcode();
    async fn get_price_and_url();
    fn parse_document(name: &str, document: &Html) -> Option<(f32, String)>;
}
