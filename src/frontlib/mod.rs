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

pub fn generate_pagination_links(total_items: usize, state: &mut State) -> String {
    let total_pages = (total_items + state.pagination.per_page - 1) / state.pagination.per_page;

    let min_visible_pages = 2;
    let offset = 1;
    let current_page = state.pagination.page;
    let ellipsis = "<a>...</a>".to_string();
    let mut pagination_html = String::new();

    // previous button
    if state.pagination.page != 0 {
        let mut new_state = state.clone();
        new_state.pagination.page -= 1;
        pagination_html.push_str(&format!(
            r#"<a href="/{}">Previous</a>"#,
            format_url_params(&new_state),
        ));
    }

    for page in 0..total_pages {
        let mut new_state = state.clone();
        new_state.pagination.page = page;
        if page < min_visible_pages || page > total_pages - min_visible_pages - 1 {
            pagination_html.push_str(&format!(
                r#"<a {} href="/{}">{}</a>"#,
                if page == current_page {
                    r#"class="active""#
                } else {
                    ""
                },
                format_url_params(&new_state),
                page,
            ));
        } else if (current_page > offset && page < current_page - offset)
            || page > current_page + offset
        {
            if !pagination_html.ends_with(&ellipsis) {
                pagination_html.push_str(&ellipsis);
            }
        } else {
            pagination_html.push_str(&format!(
                r#"<a {} href="/{}">{}</a>"#,
                if page == current_page {
                    r#"class="active""#
                } else {
                    ""
                },
                format_url_params(&new_state),
                page,
            ));
        }
    }

    // next button
    if total_pages != 0 && state.pagination.page != total_pages - 1 {
        let mut new_state = state.clone();
        new_state.pagination.page += 1;
        pagination_html.push_str(&format!(
            r#"<a href="/{}">Next</a>"#,
            format_url_params(&new_state),
        ));
    }

    pagination_html
}
