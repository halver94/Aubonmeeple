use axum::extract::Query;
use axum::response::Html;
use axum::Extension;
use axum::{extract::Form, routing::get, Router};
use std::sync::Arc;
use tokio_postgres::Client;
use tower_http::services::ServeDir;

use crate::db::{connect_db, select_count_filtered_games_from_db, select_games_from_db};
use crate::frontlib::frontpage::create_html_table;
use crate::frontlib::header::generate_header_html;
use crate::frontlib::pagination::generate_pagination_links;

use super::filter::{Filters, FiltersForm};
use super::footer::generate_footer_html;
use super::pagination::Pagination;
use super::sort::Sort;

#[derive(Debug, Clone)]
pub struct State {
    pub pagination: Pagination,
    pub filters: Filters,
    pub sort: Sort,
}

pub fn format_url_params(state: &State) -> String {
    format!(
        "?{}{}{}{}{}{}{}{}{}{}",
        format!("page={}", state.pagination.page),
        format!("&per_page={}", state.pagination.per_page),
        state
            .filters
            .city
            .as_ref()
            .map_or(String::new(), |city| format!("&city={}", city)),
        state
            .filters
            .name
            .as_ref()
            .map_or(String::new(), |name| format!("&name={}", name)),
        if state.filters.vendor.is_some() {
            format!("&vendor={}", state.filters.vendor.as_ref().unwrap())
        } else {
            String::new()
        },
        if state.filters.pro.is_some() {
            format!("&pro={}", state.filters.pro.as_ref().unwrap())
        } else {
            String::new()
        },
        if state.filters.note.is_some() {
            format!("&note={}", state.filters.note.as_ref().unwrap())
        } else {
            String::new()
        },
        if state.filters.max_price.is_some() {
            format!("&max_price={}", state.filters.max_price.as_ref().unwrap())
        } else {
            String::new()
        },
        if state.filters.min_price.is_some() {
            format!("&min_price={}", state.filters.min_price.as_ref().unwrap())
        } else {
            String::new()
        },
        format!("&sort={}", state.sort.sort),
    )
}

pub async fn root(
    pagination: Option<Query<Pagination>>,
    sort: Option<Query<Sort>>,
    filters: Option<Query<Filters>>,
    Extension(db_client): Extension<Arc<Client>>,
    filters_form: Form<FiltersForm>,
) -> Html<String> {
    let mut pagination_param = pagination.unwrap_or_default().0;
    log::debug!("FILTER FORM : {:?}", &filters_form);
    log::debug!("FILTER PARAM : {:?}", &filters);
    let mut filters_param = filters.unwrap_or_default().0;
    let sort_param = sort.unwrap_or_default().0;

    if filters_form.0.city_form.is_some() {
        let note = filters_form
            .0
            .note_form
            .unwrap()
            .parse::<f32>()
            .map_or_else(
                |e| {
                    log::error!("note parse error : {}", e);
                    None
                },
                |n| Some(n),
            );
        let max_price = filters_form
            .0
            .max_price_form
            .unwrap()
            .parse::<i32>()
            .map_or_else(
                |e| {
                    log::error!("max price parse error : {}", e);
                    None
                },
                |n| Some(n),
            );
        let min_price = filters_form
            .0
            .min_price_form
            .unwrap()
            .parse::<i32>()
            .map_or_else(
                |e| {
                    log::error!("max price parse error : {}", e);
                    None
                },
                |n| Some(n),
            );
        let pro = if filters_form.0.pro_form == Some("on".to_string()) {
            Some(true)
        } else {
            None
        };
        let city = if filters_form.0.city_form.as_ref().unwrap().is_empty() {
            None
        } else {
            filters_form.0.city_form
        };
        let name = if filters_form.0.name_form.as_ref().unwrap().is_empty() {
            None
        } else {
            filters_form.0.name_form
        };
        let vendor = if filters_form.0.vendor_form.as_ref().unwrap().is_empty() {
            None
        } else {
            filters_form.0.vendor_form
        };
        filters_param = Filters {
            city: city,
            name: name,
            vendor: vendor,
            pro: pro,
            note: note,
            max_price: max_price,
            min_price: min_price,
        }
    }

    let total_items =
        match select_count_filtered_games_from_db(&db_client, filters_param.clone()).await {
            Ok(val) => val as usize,
            Err(e) => {
                log::error!("[SERVER] error getting count filtered games : {}", e);
                0
            }
        };

    log::debug!("[SERVER] counting {} games entries from db", total_items);

    let max_page = total_items / pagination_param.per_page;
    if max_page < pagination_param.page {
        pagination_param.page = 0;
    }

    let mut state = State {
        pagination: pagination_param,
        sort: sort_param,
        filters: filters_param,
    };

    let part_games = match select_games_from_db(&db_client, &state).await {
        Ok(g) => g,
        Err(e) => {
            log::error!("[SERVER] error getting games from db : {}", e);
            return Html(String::new());
        }
    };

    log::debug!(
        "[SERVER] state {:?}, len of vec : {}",
        &state,
        part_games.games.len()
    );

    let header_html = generate_header_html();
    let filter_html = Filters::create_html(&state);
    let response_html = create_html_table(part_games, &mut state);
    let pagination_html = generate_pagination_links(total_items, &mut state);
    let footer_html = generate_footer_html();

    Html(format!(
        "{}{}{}{}{}",
        header_html, filter_html, response_html, pagination_html, footer_html
    ))
}

pub async fn set_server() {
    log::info!("[SERVER] starting server on 0.0.0.0:3000");

    let client = Arc::new(connect_db().await.unwrap());
    log::info!("[SERVER] connected with DB");

    let app = Router::new()
        .route("/", get(root).post(root))
        .nest_service("/img", ServeDir::new("img"))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(Extension(client));

    // run our app with hyper, listening globally on port 3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
