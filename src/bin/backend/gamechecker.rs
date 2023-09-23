use chrono::Duration;

use boardgame_finder::db::{
    connect_db, delete_from_all_table_with_id, select_intervalled_ids_from_oa_table_from_db,
};
use boardgame_finder::website::okkazeo::game_still_available;

pub async fn task(start_date_offset: Duration, duration: Duration) {
    let db_client = connect_db().await.unwrap();
    loop {
        let start_date = chrono::Utc::now() - start_date_offset;
        let end_date = start_date - duration;
        log::debug!(
            "[GAMECHECKER] new loop start : {:?}, end : {:?}",
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

        let loop_delay = match start_date_offset.num_days() {
            0 => Duration::hours(24),
            7 => Duration::days(7),
            _ => Duration::days(30),
        };

        let scrap_interval = if ids.len() != 0 {
            loop_delay / ids.len() as i32
        } else {
            loop_delay
        };

        println!(
            "[GAMECHECKER] checking {} games for interval (duration {:?})",
            ids.len(),
            scrap_interval
        );

        let now = chrono::Utc::now();
        for id in ids {
            log::debug!(
                "[GAMECHECKER] checking game with id {} (duration {:?})",
                id,
                scrap_interval
            );
            if !game_still_available(id as u32).await {
                // effectively removing ids that need to be removed
                log::debug!("[GAMECHECKER] removing games with id {}", id);
                if let Err(e) = delete_from_all_table_with_id(&db_client, id as i32).await {
                    log::error!("[GAMECHECKER] error deleting from db : {}", e);
                }
            }
            tokio::time::sleep(scrap_interval.to_std().unwrap()).await;
        }

        let elapsed = chrono::Utc::now() - now;
        if elapsed < loop_delay {
            log::debug!(
                "[GAMECHECKER] elapsed ({:?}) < loop_delay ({:?}), sleeping for {:?}",
                elapsed,
                loop_delay,
                loop_delay - elapsed
            );
            tokio::time::sleep((loop_delay - elapsed).to_std().unwrap()).await;
        }
    }
}

pub async fn start_game_checker() {
    log::info!("[GAMECHECKER] starting game checker thread");

    let _ = tokio::spawn(async move { task(Duration::zero(), Duration::days(7)).await });
    let _ = tokio::spawn(async move { task(Duration::days(7), Duration::days(30)).await });
    let _ = tokio::spawn(async move { task(Duration::days(30), Duration::weeks(52 * 100)).await });
    // ugly but it works..
}
