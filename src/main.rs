use feed_rs::model::Entry;
use frontend::server;
use game::{Game, OkkazeoAnnounce, Reference};
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;
use tokio::time;
use tokio_postgres::Client;
use website::agorajeux::get_agorajeux_price_and_url_by_name;
use website::knapix::get_knapix_prices;
use website::ludocortex::get_ludocortex_price_and_url;
use website::okkazeo::{
    get_atom_feed, get_okkazeo_announce_page, get_okkazeo_barcode, get_okkazeo_city,
    get_okkazeo_seller,
};
use website::philibert::get_philibert_price_and_url;
use website::ultrajeux::get_ultrajeux_price_and_url;

use log::Level;

use crate::db::{
    connect_db, insert_announce_into_db, select_game_with_id_from_db, update_game_from_db,
};
use crate::gamechecker::start_game_checker;
use crate::website::okkazeo::{
    get_okkazeo_game_image, get_okkazeo_shipping, okkazeo_is_pro_seller,
};

mod db;
mod frontend;
mod game;
mod gamechecker;
mod website;

pub async fn get_game_infos(entry: Entry) -> Box<Game> {
    log::debug!(
        "[TASK] fetching game infos for {:#?}",
        entry.title.as_ref().unwrap()
    );
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
        .parse::<f32>()
        .unwrap();

    let id = entry.id.parse::<u32>().unwrap();
    let title = entry.title.unwrap();
    let mut vec_name = title.content.split('-').collect::<Vec<&str>>();
    let extension = vec_name.pop().unwrap().trim().to_string();
    let name = vec_name.join("-").trim().to_string();
    let mut result = String::new();
    let mut inside_parentheses = false;

    for c in name.chars() {
        match c {
            '(' => inside_parentheses = true,
            ')' => inside_parentheses = false,
            _ if !inside_parentheses => result.push(c),
            _ => (),
        }
    }

    let mut game = Box::new(Game {
        okkazeo_announce: OkkazeoAnnounce {
            id,
            name,
            last_modification_date: entry.updated,
            url: entry.links.first().cloned().unwrap().href,
            extension,
            price: price,
            ..Default::default()
        },
        references: HashMap::<String, Reference>::new(),
        ..Default::default()
    });

    let re = Regex::new(r#"<img src="([^"]+)"#).unwrap();
    if let Some(captures) = re.captures(&entry.summary.unwrap().content) {
        if let Some(url) = captures.get(1) {
            game.okkazeo_announce.image = get_okkazeo_game_image(url.as_str()).await.unwrap();
        }
    }

    {
        let (document, _) = get_okkazeo_announce_page(game.okkazeo_announce.id).await;
        game.okkazeo_announce.barcode = get_okkazeo_barcode(&document);
        game.okkazeo_announce.city = get_okkazeo_city(&document);
        game.okkazeo_announce.seller = get_okkazeo_seller(&document).unwrap();
        game.okkazeo_announce.shipping = get_okkazeo_shipping(&document);
        game.okkazeo_announce.seller.is_pro = okkazeo_is_pro_seller(&document);
    }

    get_knapix_prices(&mut game).await;

    if game.references.get("philibert").is_none() {
        if let Some((price, url)) =
            get_philibert_price_and_url(&game.okkazeo_announce.name, game.okkazeo_announce.barcode)
                .await
        {
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
    if game.references.get("agorajeux").is_none() {
        if let Some((price, url)) =
            get_agorajeux_price_and_url_by_name(&game.okkazeo_announce.name).await
        {
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

    if game.references.get("ultrajeux").is_none() {
        if let Some((price, url)) =
            get_ultrajeux_price_and_url(&game.okkazeo_announce.name, game.okkazeo_announce.barcode)
                .await
        {
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

    if game.references.get("ludocortex").is_none() {
        if let Some((price, url)) =
            get_ludocortex_price_and_url(&game.okkazeo_announce.name, game.okkazeo_announce.barcode)
                .await
        {
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

    game.get_reviews().await;
    game.get_deal_advantage();

    log::debug!("[TASK] returning game {:#?}", game);
    game
}

async fn parse_game_feed(db_client: &Client) {
    log::debug!("[MAIN] parsing game feed");
    let feed = get_atom_feed().await.unwrap();

    let mut tasks = JoinSet::new();
    'outer: for entry in feed.entries {
        log::trace!("[MAIN] entry : {:#?}", entry);

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
            .parse::<f32>()
            .unwrap();

        // if same id, then it is an update
        let id = entry.id.parse::<u32>().unwrap();

        let fetched_game = select_game_with_id_from_db(db_client, id).await;
        if fetched_game.is_some() {
            let mut fetched_game = fetched_game.clone().unwrap();
            log::debug!(
                "[MAIN] updating game {}",
                fetched_game.okkazeo_announce.name
            );
            fetched_game.okkazeo_announce.last_modification_date = entry.updated;
            fetched_game.okkazeo_announce.price = price;
            if let Err(e) = update_game_from_db(db_client, &fetched_game).await {
                log::error!(
                    "erreur db, cannot update game {} : {}",
                    fetched_game.okkazeo_announce.name,
                    e
                );
            }
            continue 'outer;
        }

        tasks.spawn(async move { get_game_infos(entry).await });
    }

    while let Some(res) = tasks.join_next().await {
        let game = res.unwrap();
        log::debug!("[MAIN] got result for game {}", game.okkazeo_announce.name);

        if let Err(e) = insert_announce_into_db(db_client, &game).await {
            log::error!(
                "erreur db, cannot insert game {} : {}",
                game.okkazeo_announce.name,
                e
            );
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + 'static>> {
    log_panics::init();

    // construct a subscriber that prints formatted traces to stdout
    /*let subscriber = tracing_subscriber::FmtSubscriber::new();
        // use that subscriber to process traces emitted after this point
        tracing::subscriber::set_global_default(subscriber)?;
    */
    //this one is for vscode
    env::set_var("RUST_LOG", "boardgame_finder=debug");
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(Level::Debug.as_str()),
    )
    .init();

    //console_subscriber::init();

    let client = connect_db().await?;

    log::info!("[MAIN] starting program");
    let interval = Duration::from_secs(60 * 5); // Remplacez X par le nombre de minutes souhaité
    log::info!(
        "[MAIN] parsing game feed every {} seconds",
        interval.as_secs()
    );

    let _ = tokio::spawn(async move { server::set_server().await });
    let _ = start_game_checker().await;

    loop {
        let start = Instant::now();
        log::debug!("[MAIN] scraping time : {:#?}", start);
        parse_game_feed(&client).await;
        let duration = start.elapsed();

        if duration < interval {
            time::sleep(interval - duration).await;
        }
    }
}
