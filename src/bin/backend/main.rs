use boardgame_finder::metrics;
use boardgame_finder::website::okkazeo::get_okkazeo_csv;
use boardgame_finder::{db::select_all_ids_from_oa_table_from_db, game::get_game_infos};
use std::time::{Duration, Instant};
use tokio::time::{self};
use tokio_postgres::row;

use boardgame_finder::db::{
    connect_db, delete_from_all_table_with_id, insert_announce_into_db,
    select_game_with_id_from_db, update_game_from_db, update_sellers_nb_announces_from_db,
};

#[tokio::main]
async fn main() {
    env_logger::init();

    let backend_metrics_bind_addr =
        std::env::var("BACKEND_METRICS_ADDR").unwrap_or("127.0.0.1:3003".to_string());

    let client = connect_db().await.expect("cannot connect to DB");

    log::info!("starting program");
    let csv_fetch_interval = Duration::from_secs(60 * 60 * 3); // every 6 hours
    log::info!(
        "parsing game csv every {} seconds",
        csv_fetch_interval.as_secs()
    );

    tokio::spawn(async { metrics::run_metrics(backend_metrics_bind_addr).await });

    loop {
        let start = Instant::now();
        log::debug!("fetching time : {:?}", start);

        // fetch csv
        let rows = get_okkazeo_csv("https://www.okkazeo.com/aubonmeeple.csv".to_string())
            .await
            .unwrap();

        if rows.len() < 10 {
            let duration = start.elapsed();
            log::error!("CSV is empty !");
            time::sleep(csv_fetch_interval - duration).await;
        }

        log::info!("csv containing {} row", rows.len());
        let mut csv_ids = vec![];
        for row in &rows {
            csv_ids.push(row.id as i32);
            log::debug!("treating record : {:?}", row);
            let fetched_game = select_game_with_id_from_db(&client, row.id).await;
            match fetched_game {
                None => match get_game_infos(row.clone()).await {
                    Err(e) => log::error!("error getting game info {}", e),
                    Ok(g) => {
                        if fetched_game.is_none() {
                            if let Err(e) = insert_announce_into_db(&client, &g).await {
                                log::error!(
                                    "error db, cannot insert game {} : {}",
                                    g.okkazeo_announce.name,
                                    e
                                );
                            }
                        }
                    }
                },
                Some(mut game) => {
                    log::debug!(
                        "game {} already in DB, updating it",
                        game.okkazeo_announce.id
                    );
                    game.update_game(row.clone());
                    if let Err(e) = update_game_from_db(&client, &game).await {
                        log::error!(
                            "error db, cannot update game {} : {}",
                            game.okkazeo_announce.name,
                            e
                        );
                    }
                }
            }
        }

        let db_ids = match select_all_ids_from_oa_table_from_db(&client).await {
            Ok(ids) => {
                log::debug!("fetched {} ids", ids.len());
                ids
            }
            Err(e) => {
                log::error!("error fetching ids from db : {}", e);
                continue;
            }
        };
        let csv_set: std::collections::HashSet<i32> =
            rows.iter().cloned().map(|r| r.id as i32).collect();
        let ids_to_remove: Vec<i32> = db_ids
            .iter()
            .filter(|&x| !csv_set.contains(x))
            .cloned()
            .collect();

        log::debug!("removing {:?} games", ids_to_remove.len());
        for id in ids_to_remove {
            log::debug!("removing {} from db", id);
            if let Err(e) = delete_from_all_table_with_id(&client, id).await {
                log::error!("error deleting from db : {}", e);
            }
        }

        log::debug!(
            "updated {} sellers",
            update_sellers_nb_announces_from_db(&client).await
        );

        let duration = start.elapsed();
        log::info!("treated CSV in {:?} ", duration);
        if duration < csv_fetch_interval {
            time::sleep(csv_fetch_interval - duration).await;
        }
    }
}
