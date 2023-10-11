pub mod server;

use serde::{Deserialize, Serialize};

use self::server::{format_url_params, State};

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
    pub city: Option<String>,
    pub name: Option<String>,
    pub vendor: Option<String>,
    pub pro: Option<bool>,
    pub delivery: Option<bool>,
    pub note: Option<f32>,
    pub max_price: Option<i32>,
    pub min_price: Option<i32>,
}

// this is ugly, but otherwise the Form from axum doesnt work properly
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FiltersForm {
    pub city_form: Option<String>,
    pub name_form: Option<String>,
    pub vendor_form: Option<String>,
    pub pro_form: Option<String>,
    pub delivery_form: Option<String>,
    pub note_form: Option<String>,
    pub max_price_form: Option<String>,
    pub min_price_form: Option<String>,
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
