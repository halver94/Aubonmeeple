use boardgame_finder::game::{Game, OkkazeoAnnounce, Reference};
use boardgame_finder::metrics;
use boardgame_finder::website::agorajeux::get_agorajeux_price_and_url_by_name;
use boardgame_finder::website::knapix::get_knapix_prices;
use boardgame_finder::website::ludifolie::get_ludifolie_price_and_url_by_name;
use boardgame_finder::website::ludocortex::get_ludocortex_price_and_url;
use boardgame_finder::website::okkazeo::{
    get_atom_feed, get_okkazeo_announce_page, get_okkazeo_barcode, get_okkazeo_city,
    get_okkazeo_seller,
};
use boardgame_finder::website::philibert::get_philibert_price_and_url;
use feed_rs::model::Entry;
use lazy_static::lazy_static;
use prometheus::{register_int_counter, IntCounter};
use std::collections::HashMap;
use std::error;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;
use tokio::time;
use tokio_postgres::Client;

use crate::gamechecker::start_game_checker;
use boardgame_finder::db::{
    connect_db, insert_announce_into_db, select_game_with_id_from_db, update_game_from_db,
};

use boardgame_finder::website::okkazeo::{
    download_okkazeo_game_image, get_okkazeo_announce_extension, get_okkazeo_announce_image,
    get_okkazeo_announce_modification_date, get_okkazeo_announce_name, get_okkazeo_announce_price,
    get_okkazeo_shipping, okkazeo_is_pro_seller,
};

mod crawler;
mod gamechecker;

pub async fn get_game_infos(
    entry: Option<&Entry>,
    id: u32,
) -> Result<Box<Game>, Box<dyn error::Error + Send + Sync>> {
    log::debug!("fetching game infos for id {:?}", id);

    let image_url: String;
    let mut game = Box::new(Game {
        okkazeo_announce: OkkazeoAnnounce {
            id,
            ..Default::default()
        },
        references: HashMap::<String, Reference>::new(),
        ..Default::default()
    });

    {
        let (document, _) = get_okkazeo_announce_page(id).await;
        game.okkazeo_announce.url = format!("https://www.okkazeo.com/annonces/view/{}", id);
        game.okkazeo_announce.price = get_okkazeo_announce_price(&document)?;
        game.okkazeo_announce.extension = get_okkazeo_announce_extension(&document)?;
        game.okkazeo_announce.last_modification_date = if let Some(e) = entry {
            e.updated.unwrap()
        } else {
            get_okkazeo_announce_modification_date(&document)?
        };
        image_url = get_okkazeo_announce_image(&document)?;
        game.okkazeo_announce.barcode = get_okkazeo_barcode(&document);
        game.okkazeo_announce.city = get_okkazeo_city(&document);
        game.okkazeo_announce.shipping = get_okkazeo_shipping(&document);
        if let Some(s) = get_okkazeo_seller(&document) {
            game.okkazeo_announce.seller = s;
        }
        game.okkazeo_announce.seller.is_pro = okkazeo_is_pro_seller(&document);

        let name = get_okkazeo_announce_name(&document)?;
        let mut inside_parentheses = false;
        let mut name_result = String::new();

        for c in name.chars() {
            match c {
                '(' => inside_parentheses = true,
                ')' => inside_parentheses = false,
                _ if !inside_parentheses => name_result.push(c),
                _ => (),
            }
        }
        game.okkazeo_announce.name = name_result;
    }
    let image = download_okkazeo_game_image(&image_url).await?;
    game.okkazeo_announce.image = image;

    if let Err(e) = get_knapix_prices(&mut game).await {
        log::error!("Error gettin knapix prices : {:?}", e);
    }

    if game.references.get("philibert").is_none() {
        match get_philibert_price_and_url(
            &game.okkazeo_announce.name,
            game.okkazeo_announce.barcode,
        )
        .await
        {
            Err(e) => log::error!("error getting philibert price : {}", e),
            Ok(v) => {
                if let Some((price, url)) = v {
                    game.references.insert(
                        "philibert".to_string(),
                        Reference {
                            name: "philibert".to_string(),
                            price,
                            url,
                        },
                    );
                }
            }
        }
    }

    if game.references.get("agorajeux").is_none() {
        match get_agorajeux_price_and_url_by_name(&game.okkazeo_announce.name).await {
            Err(e) => log::error!("error getting agorajeux price : {}", e),
            Ok(v) => {
                if let Some((price, url)) = v {
                    game.references.insert(
                        "agorajeux".to_string(),
                        Reference {
                            name: "agorajeux".to_string(),
                            price,
                            url,
                        },
                    );
                }
            }
        }
    }

    if game.references.get("ludifolie").is_none() {
        match get_ludifolie_price_and_url_by_name(&game.okkazeo_announce.name).await {
            Err(e) => log::error!("error getting ludifolie price : {}", e),
            Ok(v) => {
                if let Some((price, url)) = v {
                    game.references.insert(
                        "ludifolie".to_string(),
                        Reference {
                            name: "ludifolie".to_string(),
                            price,
                            url,
                        },
                    );
                }
            }
        }
    }

    /*if game.references.get("ultrajeux").is_none() {
        match get_ultrajeux_price_and_url(
            &game.okkazeo_announce.name,
            game.okkazeo_announce.barcode,
        )
        .await
        {
            Err(e) => log::error!("error getting ultrajeux price : {}", e),
            Ok(v) => {
                if let Some((price, url)) = v {
                    game.references.insert(
                        "ultrajeux".to_string(),
                        Reference {
                            name: "ultrajeux".to_string(),
                            price,
                            url,
                        },
                    );
                }
            }
        }
    }*/

    if game.references.get("ludocortex").is_none() {
        match get_ludocortex_price_and_url(
            &game.okkazeo_announce.name,
            game.okkazeo_announce.barcode,
        )
        .await
        {
            Err(e) => log::error!("error getting ultrajeux price : {}", e),
            Ok(v) => {
                if let Some((price, url)) = v {
                    game.references.insert(
                        "ludocortex".to_string(),
                        Reference {
                            name: "ludocortex".to_string(),
                            price,
                            url,
                        },
                    );
                }
            }
        }
    }

    game.get_reviews().await;
    game.get_deal_advantage();

    log::debug!("returning game {:?}", game);
    Ok(game)
}

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
    log_panics::init();

    std::panic::set_hook(Box::new(|panic_info| {
        let backtrace = std::backtrace::Backtrace::capture();
        eprintln!("My backtrace: {:#?}", backtrace);
    }));

    env_logger::init();

    let backend_metrics_bind_addr = std::env::var("BACKEND_METRICS_ADDR").unwrap_or("127.0.0.1:3003".to_string());

    let client = connect_db().await.expect("cannot connect to DB");

    log::info!("starting program");
    let interval = Duration::from_secs(60 * 5);
    log::info!("parsing game feed every {} seconds", interval.as_secs());

    tokio::spawn(async { metrics::run_metrics(backend_metrics_bind_addr).await });
    let _ = tokio::spawn(async move { crawler::start_crawler().await });
    let _ = start_game_checker();

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
