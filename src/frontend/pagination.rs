use serde::{Deserialize, Serialize};

use super::server::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub fn generate_pagination_links(total_items: usize, state: &State) -> String {
    let params = format!(
        "sort={}{}{}{}",
        state.sort.sort,
        if state.filters.city.is_some() {
            format!("&city={}", state.filters.city.as_ref().unwrap())
        } else {
            String::new()
        },
        if state.filters.name.is_some() {
            format!("&name={}", state.filters.name.as_ref().unwrap())
        } else {
            String::new()
        },
        if state.filters.pro {
            String::from("&pro=true")
        } else {
            String::from("&pro=false")
        },
    );
    let total_pages = (total_items + state.pagination.per_page - 1) / state.pagination.per_page;

    let style = r#"<style>
                     .pagination {
                      display: inline-block;
                      }

.pagination a {
  color: black;
  float: left;
  padding: 8px 16px;
  text-decoration: none;
}

.pagination a.active {
  background-color: #4CAF50;
  color: white;
}

.pagination a:hover:not(.active) {background-color: #ddd;}
    </style>"#;

    let mut pagination_html = String::from(style);
    pagination_html.push_str(r#"<center><div class="pagination">"#);
    if state.pagination.page != 0 {
        pagination_html.push_str(&format!(
            r#"<a href="/?{}&page={}&per_page={}">Previous</a>"#,
            params,
            state.pagination.page - 1,
            state.pagination.per_page,
        ));
    }
    for page in 0..=total_pages - 1 {
        pagination_html.push_str(&format!(
            r#"<a {} href="/?{}&page={}&per_page={}">{}</a>"#,
            if page == state.pagination.page {
                r#"class="active""#
            } else {
                ""
            },
            params,
            page,
            state.pagination.per_page,
            page,
        ));
    }
    if state.pagination.page != total_pages - 1 {
        pagination_html.push_str(&format!(
            r#"<a href="/?{}&page={}&per_page={}">Next</a>"#,
            params,
            state.pagination.page + 1,
            state.pagination.per_page,
        ));
    }
    pagination_html.push_str("</div></center>");

    pagination_html
}
