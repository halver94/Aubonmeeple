use axum::extract::Query;
use axum::response::{Html, Response};
use axum::Extension;
use axum::{routing::get, Router};
use std::sync::Arc;
use tower_http::services::ServeDir;

use crate::frontend::pagination::generate_pagination_links;
use crate::game::Games;

use super::filter::Filters;
use super::footer::generate_footer_html;
use super::pagination::Pagination;
use super::sort::{self, Sort};

pub async fn root(
    pagination: Option<Query<Pagination>>,
    filters: Option<Query<Filters>>,
    sort: Option<Query<Sort>>,

    Extension(games): Extension<Arc<std::sync::Mutex<Games>>>,
) -> Html<String> {
    //debug!("pagination : {:#?}", pagination);
    let mut pagination = pagination.unwrap_or_default().0;
    let filters = filters.unwrap_or_default().0;
    let sort = sort.unwrap_or_default().0;

    let games_filtered = filters.filter(games);

    let sorted_games = Games {
        games: sort.sort(games_filtered),
    };

    let total_items = sorted_games.games.len();
    if total_items == 0 {
        return Html(Games::new().create_html_table());
    }

    let max_page = total_items / pagination.per_page;
    if max_page < pagination.page {
        pagination.page = 0;
    }

    let start_index = pagination.page * pagination.per_page;
    let mut end_index = start_index + pagination.per_page;

    if end_index > total_items {
        end_index = total_items;
    }

    let part_games: Games = Games {
        games: Vec::from_iter(sorted_games.games[start_index..end_index].iter().cloned()),
    };

    let filter_html = Filters::create_html();
    let response_html = part_games.create_html_table();
    let pagination_html = generate_pagination_links(total_items, &pagination);
    let footer_html = generate_footer_html();
    Html(format!(
        "{}{}{}{}",
        filter_html, response_html, pagination_html, footer_html
    ))
}

pub async fn set_server(games: Arc<std::sync::Mutex<Games>>) {
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
