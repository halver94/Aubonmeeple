use boardgame_finder::game::get_game_infos;
use boardgame_finder::metrics;
use boardgame_finder::website::okkazeo::get_atom_feed;
use lazy_static::lazy_static;
use prometheus::{register_int_counter, IntCounter};
use std::error;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;
use tokio::time;
use tokio_postgres::Client;

use boardgame_finder::db::{
    connect_db, insert_announce_into_db, select_game_with_id_from_db, update_game_from_db,
};

async fn parse_game_feed(db_client: &Client) -> Result<(), Box<dyn error::Error + Send + Sync>> {
    log::debug!("parsing game feed");
    let feed = get_atom_feed().await?;
    GET_ATOM_FEED.inc();

    let mut tasks = JoinSet::new();
    log::debug!("checking {} games from feed", feed.entries.len());
    'outer: for entry in feed.entries {
        log::trace!("entry : {:?}", entry);

        let price = entry
            .summary
            .as_ref()
            .unwrap()
            .content
            .split('>')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .split('€')
            .collect::<Vec<&str>>()
            .first()
            .unwrap()
            .parse::<f32>()?;

        // if same id, then it is an update
        let id = entry.id.parse::<u32>()?;

        let fetched_game = select_game_with_id_from_db(db_client, id).await;
        if fetched_game.is_some() {
            let mut fetched_game = fetched_game.clone().unwrap();
            log::debug!("updating game {}", fetched_game.okkazeo_announce.name);
            fetched_game.okkazeo_announce.last_modification_date =
                entry.updated.unwrap_or_default();
            fetched_game.okkazeo_announce.price = price;
            fetched_game.get_deal_advantage();

            if let Err(e) = update_game_from_db(db_client, &fetched_game).await {
                log::error!(
                    "error db, cannot update game {} : {}",
                    fetched_game.okkazeo_announce.name,
                    e
                );
            }
            continue 'outer;
        }

        tasks.spawn(async move { get_game_infos(Some(&entry), entry.id.parse::<u32>()?).await });
    }
    while let Some(res) = tasks.join_next().await {
        let game = res??;
        log::debug!("got result for game {}", game.okkazeo_announce.name);

        if let Err(e) = insert_announce_into_db(db_client, &game).await {
            log::error!(
                "error db, cannot insert game {} : {}",
                game.okkazeo_announce.name,
                e
            );
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let backend_metrics_bind_addr =
        std::env::var("BACKEND_METRICS_ADDR").unwrap_or("127.0.0.1:3003".to_string());

    let client = connect_db().await.expect("cannot connect to DB");

    log::info!("starting program");
    let interval = Duration::from_secs(60 * 5);
    log::info!("parsing game feed every {} seconds", interval.as_secs());

    tokio::spawn(async { metrics::run_metrics(backend_metrics_bind_addr).await });

    loop {
        let start = Instant::now();
        log::debug!("scraping time : {:?}", start);
        if let Err(e) = parse_game_feed(&client).await {
            log::error!("{}", e);
        }
        let duration = start.elapsed();

        if duration < interval {
            time::sleep(interval - duration).await;
        }
    }
}

lazy_static! {
    static ref GET_ATOM_FEED: IntCounter =
        register_int_counter!("get_atom_feed", "Number of time we get the atom feed").unwrap();
}
