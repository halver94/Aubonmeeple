#[macro_use]
extern crate log;

use feed_rs::model::Entry;
use frontend::server;
use game::{Game, Games, OkkazeoAnnounce, Reference};
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time;
use website::agorajeux::get_agorajeux_price_and_url_by_name;
use website::knapix::get_knapix_prices;
use website::ludocortex::get_ludocortex_price_and_url;
use website::okkazeo::{
    game_still_available, get_atom_feed, get_okkazeo_announce_page, get_okkazeo_barcode,
    get_okkazeo_city, get_okkazeo_seller,
};
use website::philibert::get_philibert_price_and_url;
use website::ultrajeux::get_ultrajeux_price_and_url;

use log::{debug, error, info, warn};

use crate::website::okkazeo::{get_okkazeo_game_image, get_okkazeo_shipping};

mod frontend;
mod game;
mod website;

pub async fn get_game_infos(entry: Entry) -> Box<Game> {
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

    let document = get_okkazeo_announce_page(game.okkazeo_announce.id).await;
    game.okkazeo_announce.barcode = get_okkazeo_barcode(&document).await;
    game.okkazeo_announce.city = get_okkazeo_city(&document).await;
    game.okkazeo_announce.seller = get_okkazeo_seller(&document).await.unwrap();
    game.okkazeo_announce.shipping = get_okkazeo_shipping(&document).await;

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

    game
}

async fn parse_game_feed(games: &mut Arc<std::sync::Mutex<Games>>) {
    debug!("parsing game feed");
    let feed = get_atom_feed().await.unwrap();
    'outer: for entry in feed.entries {
        //println!("entry: {:#?}", entry);

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
        for g in games.lock().unwrap().games.iter_mut() {
            if g.okkazeo_announce.id == id {
                g.okkazeo_announce.last_modification_date = entry.updated;
                g.okkazeo_announce.price = price;
                continue 'outer;
            }
        }

        // This call will make them start running in the background
        // immediately.
        let mut tasks = Vec::new();
        tasks.push(tokio::spawn(async move { get_game_infos(entry).await }));
        let mut outputs = Vec::with_capacity(tasks.len());
        for task in tasks {
            outputs.push(task.await.unwrap());
        }

        let mut locked_games = games.lock().unwrap();

        for game in outputs {
            match locked_games.games.binary_search(&game) {
                Ok(_) => {
                    debug!(
                        "game id {} already present in vec",
                        game.okkazeo_announce.id
                    )
                } // element already in vector @ `pos`
                Err(pos) => {
                    debug!("inserting game into vec : {:?}", game);
                    locked_games.games.insert(pos, game)
                }
            }
        }
    }
}

pub async fn check_list_available(games: Arc<Mutex<Games>>) {
    let interval = Duration::from_secs(60 * 60); // every hour
    loop {
        let start = Instant::now();

        // getting a list of all ID in order not to maintain a lock on the vec for too long
        let mut ids = Vec::<u32>::new();

        for game in &games.lock().unwrap().games {
            ids.push(game.okkazeo_announce.id);
        }

        let mut ids_to_remove = Vec::<u32>::new();
        for index in ids {
            if !game_still_available(index).await {
                ids_to_remove.push(index);
            }
        }

        // effectively removing ids that need to be removed
        {
            let locked_games = &mut games.lock().unwrap().games;
            locked_games.retain(|game| !ids_to_remove.contains(&game.okkazeo_announce.id));
        }

        let duration = start.elapsed();

        if duration < interval {
            tokio::time::sleep(interval - duration).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + 'static>> {
    //env_logger::init();
    info!("starting program");
    let games = Arc::new(Mutex::new(Games::new()));
    let interval = Duration::from_secs(60 * 5); // Remplacez X par le nombre de minutes souhaité
    info!("parsing game feed every {:#?} minutes", interval);

    let game_clone = games.clone();
    let game_clone2 = games.clone();
    let _ = tokio::spawn(async move { server::set_server(game_clone).await });
    let _ = tokio::spawn(async move { check_list_available(game_clone2).await });

    loop {
        let start = Instant::now();
        info!("scraping time : {:#?}", start);
        parse_game_feed(&mut games.clone()).await;
        let duration = start.elapsed();

        if duration < interval {
            time::sleep(interval - duration).await;
        }
    }
}
