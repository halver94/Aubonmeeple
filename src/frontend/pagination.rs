use serde::{Deserialize, Serialize};

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

pub fn generate_pagination_links(total_items: usize, pagination: &Pagination) -> String {
    let total_pages = (total_items + pagination.per_page - 1) / pagination.per_page;

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
    if pagination.page != 0 {
        pagination_html.push_str(&format!(
            r#"<a href="/?page={}&per_page={}">Previous</a>"#,
            pagination.page - 1,
            pagination.per_page,
        ));
    }
    for page in 0..=total_pages - 1 {
        pagination_html.push_str(&format!(
            r#"<a {} href="/?page={}&per_page={}">{}</a>"#,
            if page == pagination.page {
                r#"class="active""#
            } else {
                ""
            },
            page,
            pagination.per_page,
            page,
        ));
    }
    if pagination.page != total_pages - 1 {
        pagination_html.push_str(&format!(
            r#"<a href="/?page={}&per_page={}">Next</a>"#,
            pagination.page + 1,
            pagination.per_page,
        ));
    }
    pagination_html.push_str("</div></center>");

    pagination_html
}
