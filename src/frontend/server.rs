use axum::extract::Query;
use axum::response::Html;
use axum::Extension;
use axum::{routing::get, Router};
use std::sync::Arc;
use tower_http::services::ServeDir;

use crate::frontend::pagination::generate_pagination_links;
use crate::game::Games;

use super::filter::Filters;
use super::footer::generate_footer_html;
use super::pagination::Pagination;
use super::sort::Sort;

#[derive(Debug, Clone)]
pub struct State {
    pub pagination: Pagination,
    pub filters: Filters,
    pub sort: Sort,
}

pub async fn root(
    pagination: Option<Query<Pagination>>,
    filters: Option<Query<Filters>>,
    sort: Option<Query<Sort>>,

    Extension(games): Extension<Arc<std::sync::Mutex<Games>>>,
) -> Html<String> {
    //debug!("pagination : {:#?}", pagination);
    let mut pagination_param = pagination.unwrap_or_default().0;
    let filters_param = filters.unwrap_or_default().0;
    let sort_param = sort.unwrap_or_default().0;

    let games_filtered = filters_param.filter(games);

    let sorted_games = Games {
        games: sort_param.sort(games_filtered),
    };

    let total_items = sorted_games.games.len();
    if total_items == 0 {
        return Html(String::new());
    }

    let max_page = total_items / pagination_param.per_page;
    if max_page < pagination_param.page {
        pagination_param.page = 0;
    }

    let start_index = pagination_param.page * pagination_param.per_page;
    let mut end_index = start_index + pagination_param.per_page;

    if end_index > total_items {
        end_index = total_items;
    }

    let part_games: Games = Games {
        games: Vec::from_iter(sorted_games.games[start_index..end_index].iter().cloned()),
    };

    let state = State {
        pagination: pagination_param,
        sort: sort_param,
        filters: filters_param,
    };

    log::debug!("[SERVER] state {:#?}", state);
    let filter_html = Filters::create_html(&state);
    let response_html = part_games.create_html_table(&state);
    let pagination_html = generate_pagination_links(total_items, &state);
    let footer_html = generate_footer_html();
    Html(format!(
        "{}{}{}{}",
        filter_html, response_html, pagination_html, footer_html
    ))
}

pub async fn set_server(games: Arc<std::sync::Mutex<Games>>) {
    log::info!("[SERVER] starting server on 0.0.0.0:3000");
    let app = Router::new()
        .route("/", get(root))
        .nest_service("/img", ServeDir::new("img"))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(Extension(games));

    // run our app with hyper, listening globally on port 3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
