use chrono::Duration;

use crate::{
    db::{connect_db, delete_from_all_table_with_id, select_intervalled_ids_from_oa_table_from_db},
    website::okkazeo::game_still_available,
};

pub async fn task(
    scrap_interval: tokio::time::Duration,
    start_date_offset: Duration,
    duration: Duration,
) {
    let db_client = connect_db().await.unwrap();
    loop {
        let start_date = chrono::Utc::now() + start_date_offset;
        let end_date = start_date + duration;
        log::debug!(
            "[GAMECHECKER] new loop start : {}, end : {:#?}",
            start_date_offset,
            duration
        );
        let ids =
            match select_intervalled_ids_from_oa_table_from_db(&db_client, start_date, end_date)
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    log::error!("{}", e);
                    vec![]
                }
            };

        println!(
            "[GAMECHECKER] checking {} games for interval (duration {:#?}",
            ids.len(),
            scrap_interval
        );
        for id in ids {
            log::debug!(
                "[GAMECHECKER] checking game with id {} (duration {:#?}",
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
            tokio::time::sleep(scrap_interval).await;
        }
    }
}

pub async fn start_game_checker() {
    log::info!("[GAMECHECKER] starting game checker thread");

    let small_interval = tokio::time::Duration::from_secs(20); // for game < 1 week
    let medium_interval = tokio::time::Duration::from_secs(30); // for 1 week < game < 1 month
    let big_interval = tokio::time::Duration::from_secs(60); // for game > 1 month

    let _ = tokio::spawn(
        async move { task(small_interval, Duration::zero(), Duration::days(7)).await },
    );
    let _ =
        tokio::spawn(
            async move { task(medium_interval, Duration::days(7), Duration::days(30)).await },
        );
    let _ =
        tokio::spawn(
            async move { task(big_interval, Duration::days(30), Duration::max_value()).await },
        );
}
