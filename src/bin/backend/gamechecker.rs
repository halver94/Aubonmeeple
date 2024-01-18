use chrono::Duration;
use lazy_static::lazy_static;
use rand::Rng;

use boardgame_finder::db::{
    connect_db, delete_from_all_table_with_id, select_intervalled_ids_from_oa_table_from_db,
};
use boardgame_finder::website::okkazeo::game_still_available;
use prometheus::{register_int_counter, IntCounter};

pub async fn task(start_date_offset: Duration, duration: Duration) {
    log::info!(
        "starting game checker task with start_date {} and duration {}",
        start_date_offset,
        duration
    );
    let db_client = connect_db().await.unwrap();
    loop {
        let start_date = chrono::Utc::now() - start_date_offset;
        let end_date = start_date - duration;
        log::debug!(
            "new loop start : {:?}, end : {:?}",
            start_date_offset,
            duration
        );
        let ids =
            match select_intervalled_ids_from_oa_table_from_db(&db_client, end_date, start_date)
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    log::error!("{}", e);
                    vec![]
                }
            };

        for id in ids {
            log::debug!("checking game with id {})", id,);
            GAMECHECKER_CHECKED_GAME.inc();
            if !game_still_available(id as u32).await {
                // effectively removing ids that need to be removed
                log::debug!("removing games with id {}", id);
                if let Err(e) = delete_from_all_table_with_id(&db_client, id).await {
                    GAMECHECKER_REMOVED_GAME.inc();
                    log::error!("error deleting from db : {}", e);
                }
            }
            let sleep_duration = {
                let mut rng = rand::thread_rng();
                Duration::seconds(rng.gen_range(1..=7))
            };
            tokio::time::sleep(sleep_duration.to_std().unwrap()).await;
        }
    }
}

pub fn start_game_checker() {
    log::info!("starting game checker thread");

    let _ = tokio::spawn(async move { task(Duration::zero(), Duration::days(3)).await });
    let _ = tokio::spawn(async move { task(Duration::days(3), Duration::days(10)).await });
    let _ = tokio::spawn(async move { task(Duration::days(10), Duration::weeks(52 * 100)).await });
    // ugly but it works..
}

lazy_static! {
    static ref GAMECHECKER_CHECKED_GAME: IntCounter = register_int_counter!(
        "gamechecker_checked_game",
        "Number of game availability checked"
    )
    .unwrap();
    static ref GAMECHECKER_REMOVED_GAME: IntCounter =
        register_int_counter!("gamechecker_removed_game", "Number of game removed").unwrap();
}
