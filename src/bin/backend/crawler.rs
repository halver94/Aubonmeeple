use std::time::Duration;

use tokio::time;

use boardgame_finder::db::{
    connect_db, insert_announce_into_db, select_game_with_id_from_db, update_game_from_db,
};
use boardgame_finder::website::okkazeo::get_games_from_page;

use crate::get_game_infos;

pub async fn start_crawler() {
    log::info!("starting crawler thread");

    let db_client = connect_db().await.unwrap();

    let interval = Duration::from_secs(20);
    let mut interval_stream = time::interval(interval);

    let mut page = 1;
    loop {
        match get_games_from_page(page).await {
            Err(e) => log::error!("error getting game from page {} :{}", page, e),
            Ok(v) => {
                log::debug!("fetching {} games for page {}", v.len(), page);
                for id in v {
                    let fetched_game = select_game_with_id_from_db(&db_client, id).await;
                    match get_game_infos(None, id).await {
                        Err(e) => log::error!("{}", e),
                        Ok(g) => {
                            if fetched_game.is_none() {
                                if let Err(e) = insert_announce_into_db(&db_client, &g).await {
                                    log::error!(
                                        "erreur db, cannot insert game {} : {}",
                                        g.okkazeo_announce.name,
                                        e
                                    );
                                }
                            } else {
                                if let Err(e) = update_game_from_db(&db_client, &g).await {
                                    log::error!(
                                        "erreur db, cannot update game {} : {}",
                                        g.okkazeo_announce.name,
                                        e
                                    );
                                }
                            }
                        }
                    }
                    interval_stream.tick().await;
                }
            }
        }
        page += 1;
    }
}
