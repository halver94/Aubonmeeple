pub mod server;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    pub sort: String,
}

impl Default for Sort {
    fn default() -> Self {
        Self {
            sort: String::from("updated"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Filters {
    pub date: Option<String>,
    pub city: Option<String>,
    pub name: Option<String>,
    pub vendor: Option<String>,
    pub pro: Option<bool>,
    pub delivery: Option<bool>,
    pub note: Option<f32>,
    pub max_price: Option<i32>,
    pub min_price: Option<i32>,
    pub type_game: Option<bool>,
    pub type_game_ext: Option<bool>,
    pub type_ext: Option<bool>,
    pub type_misc: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Fildset {
    pub red_form: Option<String>,
    pub orange_form: Option<String>,
}

// this is ugly, but otherwise the Form from axum doesnt work properly
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FiltersForm {
    pub date_form: Option<String>,
    pub city_form: Option<String>,
    pub name_form: Option<String>,
    pub vendor_form: Option<String>,
    pub pro_form: Option<String>,
    pub delivery_form: Option<String>,
    pub note_form: Option<String>,
    pub max_price_form: Option<String>,
    pub min_price_form: Option<String>,
    pub per_page_form: Option<String>,
    pub type_game_form: Option<String>,
    pub type_game_ext_form: Option<String>,
    pub type_ext_form: Option<String>,
    pub type_misc_form: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Pagination {
    pub per_page: usize,
    pub page: usize,
}
impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 0,
            per_page: 25,
        }
    }
}
