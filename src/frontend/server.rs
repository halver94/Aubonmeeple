use axum::extract::Query;
use axum::response::Html;
use axum::Extension;
use axum::{routing::get, Router};
use std::sync::Arc;
use tokio_postgres::Client;
use tower_http::services::ServeDir;

use crate::db::{connect_db, select_count_filtered_games_from_db, select_games_from_db};
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

pub fn create_html_table(games: Games, state: &State) -> String {
    let params = format!(
        "page={}&per_page={}{}{}",
        state.pagination.page,
        state.pagination.per_page,
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
    );

    let mut table = String::new();
    table.push_str(
        "<style>
                    table {
                        border-collapse: collapse;
                        width: 100%;
                    }
                    th, td {
                        border: 1px solid black;
                        padding: 8px;
                        text-align: center;
                    }
                    th {
                        background-color: lightgray;
                    }
                   
                    .flex-container {
                        display: flex;
                        align-items: center; /* Alignement vertical */
                    }
                    .flex-container img {
                        margin-right: 10px; /* Espacement entre l'image et le texte */
                    }

                    </style>",
    );
    table.push_str("<table>");
    table.push_str(
        format!(
            "{}{}{}{}{}",
            r#"<tr>
            <th>Updated <button onclick="window.location.href='/?"#,
            params,
            r#"&sort=updated';">Sort</button></th>
            <th>Name</th>
            <th>City</th>
            <th>Seller</th>
            <th>Shipping</th>
            <th>Deal <button onclick="window.location.href='/?"#,
            params,
            r#"&sort=deal';">Sort</button></th>
            <th>Okkazeo</th>
            <th>Shops</th>
            <th>Note</th>
        </tr>"#
        )
        .as_str(),
    );

    for game in games.games.iter() {
        table.push_str("<tr>");
        table.push_str(&format!(
            "<td>{}</td>",
            game.okkazeo_announce
                .last_modification_date
                .unwrap()
                .format("%d/%m/%Y %H:%M")
        ));
        table.push_str(&format!(
            "<td>
                    <div class=\"flex-container\">
                        <img src=\"{}\" alt=\"fail\" width=\"100\" height=\"100\" />
                        {}<br>({})
                    </div>
                </td>",
            game.okkazeo_announce.image,
            game.okkazeo_announce.name,
            game.okkazeo_announce.extension
        ));
        table.push_str(&format!(
            "<td>{}</td>",
            game.okkazeo_announce.city.clone().unwrap_or(String::new())
        ));
        table.push_str(&format!(
            "<td><a href=\"{}\">{} {}<br>({} announces)</a></td>",
            game.okkazeo_announce.seller.url,
            game.okkazeo_announce.seller.name,
            if game.okkazeo_announce.seller.is_pro {
                "- PRO"
            } else {
                ""
            },
            game.okkazeo_announce.seller.nb_announces
        ));

        table.push_str("<td>");
        for (key, val) in game.okkazeo_announce.shipping.iter() {
            table.push_str(&format!("- {} : {}€<br>", key, val));
        }
        table.push_str("</td>");

        if game.deal.deal_price != 0 {
            table.push_str(&format!(
                "<td style=\"color: {}\">{}{}€ ({}{}%)</td>",
                if game.deal.deal_price < 0 {
                    "green"
                } else {
                    "red"
                },
                if game.deal.deal_price >= 0 { "+" } else { "" },
                game.deal.deal_price,
                if game.deal.deal_percentage > 0 {
                    "+"
                } else {
                    ""
                },
                game.deal.deal_percentage,
            ));
        } else {
            table.push_str("<td>-</td>");
        }

        table.push_str(&format!(
            "<td><a href=\"{}\">{} &euro;</a></td>",
            game.okkazeo_announce.url, game.okkazeo_announce.price,
        ));

        table.push_str("<td>");
        if game.references.is_empty() {
            table.push_str("-");
        } else {
            for val in game.references.values() {
                table.push_str(&format!(
                    "<a href=\"{}\">{} : {} &euro;</a><br>",
                    val.url, val.name, val.price,
                ));
            }
        }
        table.push_str("</td>");

        if game.review.average_note == 0.0 {
            table.push_str("<td>-</td>");
        } else {
            table.push_str(&format!(
                "<td>Average note : {:.2}<br><br>",
                game.review.average_note,
            ));

            for val in game.review.reviews.values() {
                table.push_str(&format!(
                    "<a href=\"{}\">{}: {} ({} reviews)</a><br>",
                    val.url, val.name, val.note, val.number
                ));
            }
            table.push_str("</td>");
        }
        table.push_str("</tr>");
    }

    table.push_str("</table>");
    table
}

pub async fn root(
    pagination: Option<Query<Pagination>>,
    filters: Option<Query<Filters>>,
    sort: Option<Query<Sort>>,

    Extension(db_client): Extension<Arc<Client>>,
) -> Html<String> {
    let mut pagination_param = pagination.unwrap_or_default().0;
    let filters_param = filters.unwrap_or_default().0;
    let sort_param = sort.unwrap_or_default().0;

    let total_items =
        match select_count_filtered_games_from_db(&db_client, filters_param.clone()).await {
            Ok(val) => val as usize,
            Err(e) => {
                log::error!("[SERVER] error getting count filtered games : {}", e);
                0
            }
        };

    log::debug!("[SERVER] counting {} games entries from db", total_items);

    if total_items == 0 {
        return Html(String::new());
    }

    let max_page = total_items / pagination_param.per_page;
    if max_page < pagination_param.page {
        pagination_param.page = 0;
    }

    let state = State {
        pagination: pagination_param,
        sort: sort_param,
        filters: filters_param.clone(),
    };

    let part_games = match select_games_from_db(&db_client, &state).await {
        Ok(g) => g,
        Err(e) => {
            log::error!("[SERVER] error getting games from db : {}", e);
            return Html(String::new());
        }
    };

    log::debug!(
        "[SERVER] state {:#?}, len of vec : {}",
        &state,
        part_games.games.len()
    );
    let filter_html = Filters::create_html(&state);
    let response_html = create_html_table(part_games, &state);
    let pagination_html = generate_pagination_links(total_items, &state);
    let footer_html = generate_footer_html();
    Html(format!(
        "{}{}{}{}",
        filter_html, response_html, pagination_html, footer_html
    ))
}

pub async fn set_server() {
    log::info!("[SERVER] starting server on 0.0.0.0:3000");

    let client = Arc::new(connect_db().await.unwrap());
    log::info!("[SERVER] connected with DB");

    let app = Router::new()
        .route("/", get(root))
        .nest_service("/img", ServeDir::new("img"))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(Extension(client));

    // run our app with hyper, listening globally on port 3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
