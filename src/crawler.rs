use std::time::Duration;

use tokio::time;

use crate::{
    db::{connect_db, insert_announce_into_db, select_game_with_id_from_db},
    get_game_infos,
    website::okkazeo::get_games_from_page,
};

pub async fn start_crawler() {
    log::info!("starting crawler thread");

    let db_client = connect_db().await.unwrap();

    let interval = Duration::from_secs(60);
    let mut interval_stream = time::interval(interval);

    let mut page = 1;
    loop {
        log::debug!("crawler tick");
        match get_games_from_page(page).await {
            Err(e) => log::error!("error getting game from page {} :{}", page, e),
            Ok(v) => {
                log::debug!("fetching {} games for page {}", v.len(), page);
                for id in v {
                    let fetched_game = select_game_with_id_from_db(&db_client, id).await;
                    if fetched_game.is_none() {
                        match get_game_infos(None, id).await {
                            Err(e) => log::error!("{}", e),
                            Ok(g) => {
                                if let Err(e) = insert_announce_into_db(&db_client, &g).await {
                                    log::error!(
                                        "erreur db, cannot insert game {} : {}",
                                        g.okkazeo_announce.name,
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        //interval_stream.tick().await;
        page += 1;
    }
    // every 2min get next page
    // parse page and games
    // insert into DB
}
