use axum::extract::Query;
use axum::response::Html;
use axum::Extension;
use axum::{routing::get, Router};
use std::sync::Arc;

use crate::frontend::pagination::generate_pagination_links;
use crate::game::Games;

use super::filter::Filters;
use super::pagination::Pagination;

pub async fn root(
    pagination: Option<Query<Pagination>>,
    filters: Option<Query<Filters>>,
    Extension(games): Extension<Arc<std::sync::Mutex<Games>>>,
) -> Html<String> {
    println!("pagination : {:#?}", pagination);
    let pagination = pagination.unwrap_or_default().0;
    let filters = filters.unwrap_or_default().0;

    let games_filtered = filters.filter(games);
    let total_items = games_filtered.len();
    if total_items == 0 {
        return Html(Games::new().create_html_table());
    }

    let start_index = pagination.page * pagination.per_page;
    let mut end_index = start_index + pagination.per_page;

    if end_index > total_items {
        end_index = total_items - 1;
    }

    let part_games: Games = Games {
        games: Vec::from_iter(games_filtered[start_index..end_index].iter().cloned()),
    };

    let response_html = part_games.create_html_table();
    let pagination_html = generate_pagination_links(total_items, &pagination);
    Html(format!("{}{}", response_html, pagination_html))
}

pub async fn set_server(games: Arc<std::sync::Mutex<Games>>) {
    let app = Router::new().route("/", get(root)).layer(Extension(games));

    // run our app with hyper, listening globally on port 3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
