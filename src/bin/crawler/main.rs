use boardgame_finder::game::get_game_infos;
use boardgame_finder::metrics;

use lazy_static::lazy_static;
use prometheus::{register_int_counter, IntCounter};

use boardgame_finder::db::{
    connect_db, insert_announce_into_db, select_game_with_id_from_db, update_game_from_db,
};
use boardgame_finder::website::okkazeo::get_games_from_page;

#[tokio::main]
async fn main() {
    env_logger::init();

    let backend_metrics_bind_addr =
        std::env::var("BACKEND_METRICS_ADDR").unwrap_or("127.0.0.1:3003".to_string());

    log::info!("starting program");
    tokio::spawn(async { metrics::run_metrics(backend_metrics_bind_addr).await });

    let db_client = connect_db().await.unwrap();

    let mut page = 1;
    loop {
        CRAWLER_PAGE_CRAWLED.inc();
        match get_games_from_page(page).await {
            Err(e) => {
                log::error!(
                    "error getting game from page {} :{}, exiting crawler",
                    page,
                    e
                );
                break;
            }
            Ok(v) => {
                log::info!("fetching {} games for page {}", v.len(), page);
                for id in v {
                    CRAWLER_GAME_CRAWLED.inc();
                    let fetched_game = select_game_with_id_from_db(&db_client, id).await;
                    match get_game_infos(None, id).await {
                        Err(e) => log::error!("{}", e),
                        Ok(g) => {
                            if fetched_game.is_none() {
                                if let Err(e) = insert_announce_into_db(&db_client, &g).await {
                                    log::error!(
                                        "error db, cannot insert game {} : {}",
                                        g.okkazeo_announce.name,
                                        e
                                    );
                                }
                            } else if let Err(e) = update_game_from_db(&db_client, &g).await {
                                log::error!(
                                    "error db, cannot update game {} : {}",
                                    g.okkazeo_announce.name,
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }
        page += 1;
    }
    log::info!("exiting crawler");
}

lazy_static! {
    static ref CRAWLER_PAGE_CRAWLED: IntCounter =
        register_int_counter!("crawler_page_crawled", "Number of page crawled").unwrap();
    static ref CRAWLER_GAME_CRAWLED: IntCounter =
        register_int_counter!("crawler_game_crawled", "Number of game crawled").unwrap();
}
