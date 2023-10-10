use std::collections::HashMap;

use chrono::{DateTime, Utc};
use tokio_postgres::{Client, Error, NoTls, Row};

use crate::frontlib::server::State;
use crate::{
    frontlib::Filters,
    game::{Deal, Game, Games, OkkazeoAnnounce, Reference, Review, Reviewer, Seller},
};

pub async fn connect_db() -> Result<Client, Error> {
    let db_url = "postgres://scrapy:scrapyscrapy@localhost/scraper";

    log::info!("[DB] connecting to DB");
    let (client, connection) = tokio_postgres::connect(db_url, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            log::error!("Erreur de connexion: {}", e);
        }
    });

    Ok(client)
}

pub async fn delete_from_all_table_with_id(db_client: &Client, id: i32) -> Result<(), Error> {
    db_client
        .execute("DELETE FROM deal WHERE deal_oa_id = $1", &[&id])
        .await?;

    db_client
        .execute("DELETE FROM shipping WHERE ship_oa_id = $1", &[&id])
        .await?;

    db_client
        .execute("DELETE FROM reference WHERE ref_oa_id = $1", &[&id])
        .await?;

    db_client
        .execute("DELETE FROM reviewer WHERE reviewer_oa_id = $1", &[&id])
        .await?;

    db_client
        .execute("DELETE FROM okkazeo_announce WHERE oa_id = $1", &[&id])
        .await?;

    Ok(())
}

pub async fn insert_into_okkazeo_announce_table(
    db_client: &Client,
    game: &Box<Game>,
) -> Result<(), Error> {
    let okkazeo_insert_req = format!(
        r#"INSERT INTO okkazeo_announce ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
        "oa_id",
        "oa_last_modification_date",
        "oa_name",
        "oa_image",
        "oa_price",
        "oa_url",
        "oa_extension",
        "oa_seller",
        "oa_barcode",
        "oa_city",
    );
    let _ = db_client
        .query(
            &okkazeo_insert_req,
            &[
                &(game.okkazeo_announce.id as i32),
                &game.okkazeo_announce.last_modification_date,
                &game.okkazeo_announce.name,
                &game.okkazeo_announce.image,
                &game.okkazeo_announce.price,
                &game.okkazeo_announce.url,
                &game.okkazeo_announce.extension,
                &(game.okkazeo_announce.seller.id as i32),
                &(game.okkazeo_announce.barcode.unwrap_or_default() as i64),
                &game
                    .okkazeo_announce
                    .city
                    .as_ref()
                    .unwrap_or(&String::from("")),
            ],
        )
        .await?;

    Ok(())
}

pub async fn insert_into_shipping_table(
    db_client: &Client,
    id: i32,
    shipping: &HashMap<String, f32>,
) -> Result<(), Error> {
    let seller_insert_req = format!(
        r#"INSERT INTO shipping ({}, {}, {}) VALUES ($1, $2, $3)"#,
        "ship_oa_id", "ship_shipper", "ship_price",
    );

    for (key, value) in shipping.iter() {
        let _ = db_client
            .query(&seller_insert_req, &[&id, &key, &value])
            .await?;
    }

    Ok(())
}

pub async fn insert_into_seller_table(db_client: &Client, seller: &Seller) -> Result<(), Error> {
    log::debug!("insertin into seller table");
    let seller_insert_req = format!(
        r#"INSERT INTO seller ({}, {}, {}, {}, {}) VALUES ($1, $2, $3, $4, $5)"#,
        "seller_id", "seller_name", "seller_url", "seller_nb_announces", "seller_is_pro",
    );
    let _ = db_client
        .query(
            &seller_insert_req,
            &[
                &(seller.id as i32),
                &seller.name,
                &seller.url,
                &(seller.nb_announces as i32),
                &seller.is_pro,
            ],
        )
        .await?;

    Ok(())
}
pub async fn insert_into_deal_table(db_client: &Client, id: i32, deal: &Deal) -> Result<(), Error> {
    let deal_insert_req = format!(
        r#"INSERT INTO deal ({}, {}, {}) VALUES ($1, $2, $3)"#,
        "deal_oa_id", "deal_price", "deal_percentage",
    );
    let _ = db_client
        .query(
            &deal_insert_req,
            &[&id, &deal.deal_price, &deal.deal_percentage],
        )
        .await?;

    Ok(())
}

pub async fn insert_into_reference_table(
    db_client: &Client,
    id: i32,
    references: &HashMap<String, Reference>,
) -> Result<(), Error> {
    let references_insert_req = format!(
        r#"INSERT INTO reference ({}, {}, {}, {}) VALUES ($1, $2, $3, $4)"#,
        "ref_oa_id", "ref_name", "ref_price", "ref_url",
    );

    for val in references.values() {
        let _ = db_client
            .query(
                &references_insert_req,
                &[&id, &val.name, &val.price, &val.url],
            )
            .await?;
    }
    Ok(())
}

pub async fn insert_into_reviewer_table(
    db_client: &Client,
    id: i32,
    reviewers: &HashMap<std::string::String, Reviewer>,
) -> Result<(), Error> {
    let references_insert_req = format!(
        r#"INSERT INTO reviewer ({}, {}, {}, {}, {}) VALUES ($1, $2, $3, $4, $5)"#,
        "reviewer_oa_id", "reviewer_name", "reviewer_url", "reviewer_note", "reviewer_number",
    );

    for val in reviewers.values() {
        let _ = db_client
            .query(
                &references_insert_req,
                &[&id, &val.name, &val.url, &val.note, &(val.number as i32)],
            )
            .await?;
    }
    Ok(())
}

pub async fn insert_announce_into_db(db_client: &Client, game: &Box<Game>) -> Result<(), Error> {
    log::debug!("inserting {} into DB ", game.okkazeo_announce.name);
    //chck if seller already hs announes, if yes update, if not insert
    if check_if_seller_in_db(db_client, game.okkazeo_announce.seller.id as i32).await? > 0 {
        log::debug!("seller {:?} present in DB", game.okkazeo_announce.seller);
        update_seller_table_from_db(db_client, &game.okkazeo_announce.seller).await?;
    } else {
        log::debug!(
            "seller {:?} not present in DB",
            game.okkazeo_announce.seller
        );
        insert_into_seller_table(db_client, &game.okkazeo_announce.seller).await?;
    }
    let id = game.okkazeo_announce.id as i32;
    insert_into_okkazeo_announce_table(db_client, game).await?;
    insert_into_shipping_table(db_client, id, &game.okkazeo_announce.shipping).await?;
    insert_into_deal_table(db_client, id, &game.deal).await?;
    insert_into_reference_table(db_client, id, &game.references).await?;
    insert_into_reviewer_table(db_client, id, &game.review.reviews).await?;

    Ok(())
}

pub async fn update_seller_table_from_db(db_client: &Client, seller: &Seller) -> Result<(), Error> {
    log::debug!("updating seller table");
    let references_insert_req = format!(
        r#"UPDATE seller SET {} = $1 WHERE {} = $2"#,
        "seller_nb_announces", "seller_id",
    );

    let _ = db_client
        .query(
            &references_insert_req,
            &[&(seller.nb_announces as i32), &(seller.id as i32)],
        )
        .await?;
    Ok(())
}
pub async fn update_okkazeo_announce_table_from_db(
    db_client: &Client,
    game: &Game,
) -> Result<(), Error> {
    let references_insert_req = format!(
        r#"UPDATE okkazeo_announce SET {} = $1, {} = $2 WHERE {} = $3"#,
        "oa_last_modification_date", "oa_price", "oa_id",
    );

    let _ = db_client
        .query(
            &references_insert_req,
            &[
                &game.okkazeo_announce.last_modification_date,
                &game.okkazeo_announce.price,
                &(game.okkazeo_announce.id as i32),
            ],
        )
        .await?;
    Ok(())
}

pub async fn update_deal_table(db_client: &Client, id: i32, deal: &Deal) -> Result<(), Error> {
    let deal_insert_req = format!(
        r#"UPDATE deal SET {} = $1, {} = $2 WHERE {} = $3"#,
        "deal_price", "deal_percentage", "deal_oa_id",
    );
    let _ = db_client
        .query(
            &deal_insert_req,
            &[&deal.deal_price, &deal.deal_percentage, &id],
        )
        .await?;

    Ok(())
}
pub async fn update_game_from_db(db_client: &Client, game: &Game) -> Result<(), Error> {
    update_okkazeo_announce_table_from_db(db_client, game).await?;
    update_deal_table(db_client, game.okkazeo_announce.id as i32, &game.deal).await?;
    Ok(())
}

pub async fn craft_game_from_row(db_client: &Client, row: Row) -> Result<Game, Error> {
    let id: i32 = row.try_get("oa_id")?;
    let nb_announces: i32 = row.try_get("seller_nb_announces")?;
    let seller_id: i32 = row.try_get("seller_id")?;

    let game = Game {
        okkazeo_announce: OkkazeoAnnounce {
            id: id as u32,
            name: row.try_get("oa_name")?,
            image: row.try_get("oa_image")?,
            price: row.try_get("oa_price")?,
            url: row.try_get("oa_url")?,
            extension: row.try_get("oa_extension").unwrap_or_default(),
            shipping: select_shipping_from_db(db_client, id).await?,
            seller: Seller {
                id: seller_id as u32,
                name: row.try_get("seller_name")?,
                url: row.try_get("seller_url")?,
                nb_announces: nb_announces as u32,
                is_pro: row.try_get("seller_is_pro")?,
            },
            barcode: match row.try_get::<&str, i64>("oa_barcode") {
                Ok(v) => Some(v as u64),
                Err(_) => None,
            },
            city: row.try_get("oa_city")?,
            last_modification_date: row.try_get("oa_last_modification_date")?,
        },
        references: select_references_from_db(db_client, id).await?,
        review: select_reviews_from_db(db_client, id).await?,
        deal: Deal {
            deal_price: row.try_get("deal_price")?,
            deal_percentage: row.try_get("deal_percentage")?,
        },
    };

    Ok(game)
}

pub async fn select_game_with_id_from_db(db_client: &Client, id: u32) -> Option<Game> {
    log::debug!("[DB] select game with id from db : {}", id);
    let select_req = format!(
        "SELECT *
                FROM okkazeo_announce oa
                JOIN deal d on d.deal_oa_id = oa.oa_id
                JOIN seller s on s.seller_id = oa.oa_seller
                WHERE oa.oa_id = $1"
    );

    let res = match db_client.query(&select_req, &[&(id as i32)]).await {
        Ok(r) => r,
        Err(e) => {
            log::error!("select game with id erro : {}", e);
            return None;
        }
    };

    let row = res.into_iter().next()?;

    match craft_game_from_row(db_client, row).await {
        Ok(game) => Some(game),
        Err(e) => {
            log::error!("[DB] craft game from row error for id {} : {}", id, e);
            None
        }
    }
}

pub async fn select_games_from_db(db_client: &Client, state: &State) -> Result<Games, Error> {
    let now = chrono::Utc::now();
    let order_by = match state.sort.sort.as_str() {
        "price" => "d.deal_price ASC",
        "percent" => "d.deal_percentage ASC",
        _ => "oa.oa_last_modification_date DESC",
    };

    let select_req = format!(
        "SELECT 
            oa.oa_name,
            oa.oa_last_modification_date,
            oa.oa_id,
            oa.oa_price,
            oa.oa_url,
            oa.oa_extension,
            oa.oa_image,
            oa.oa_city,
            s.seller_id,
            s.seller_name,
            s.seller_url,
            s.seller_is_pro,
            s.seller_nb_announces,
            d.deal_price,
            d.deal_percentage
         FROM okkazeo_announce oa
                JOIN deal d on d.deal_oa_id = oa.oa_id
                LEFT JOIN reviewer r on r.reviewer_oa_id = oa.oa_id
                JOIN seller s on s.seller_id = oa.oa_seller
                WHERE oa.oa_id IN (
                    SELECT oa.oa_id
                    FROM okkazeo_announce oa
                    LEFT JOIN reviewer r on r.reviewer_oa_id = oa.oa_id
                    JOIN seller s on s.seller_id = oa.oa_seller
                    WHERE unaccent(oa.oa_name) ilike unaccent($1) AND unaccent(oa.oa_city) ilike unaccent($2)
                    AND unaccent(s.seller_name) ilike unaccent($3)
                    AND oa.oa_price > $4
                    AND oa.oa_price < $5
                    {}
                    GROUP BY oa.oa_id
                    {}
                )
                GROUP BY
                    oa.oa_last_modification_date,
                    oa.oa_name,
                    oa.oa_id,
                    oa.oa_price,
                    oa.oa_url,
                    oa.oa_extension,
                    oa.oa_barcode,
                    oa.oa_image,
                    oa.oa_city,
                    s.seller_id,
                    s.seller_name,
                    s.seller_url,
                    s.seller_is_pro,
                    s.seller_nb_announces,
                    d.deal_price,
                    d.deal_percentage
                ORDER BY {} LIMIT $6 OFFSET $7;",
        if state.filters.pro.is_some() {
            "AND NOT s.seller_is_pro"
        } else {
            ""
        },
        if state.filters.note.is_some() {
            format!(
                "HAVING SUM(CASE WHEN r.reviewer_number > 0 THEN r.reviewer_note * r.reviewer_number ELSE 0 END) / SUM(CASE WHEN r.reviewer_number > 0
                 THEN r.reviewer_number ELSE 1 END) > {}",
                //"HAVING AVG(r.reviewer_note* r.reviewer_number) / SUM(r.reviewer_number) > {}",
                state.filters.note.unwrap()
            )
        } else {
            "".to_string()
        },
        order_by
    );
    log::debug!(
        "Took {} befoe req",
        (chrono::Utc::now() - now).num_milliseconds()
    );

    // this is a trick, if city filter is a number, it means that we're
    // looking for postcode. Okkazeo format for city is : "city (postcode)"
    let mut match_start = "";
    if state.filters.city.is_some() && state.filters.city.as_ref().unwrap().parse::<i32>().is_ok() {
        match_start = "(";
    }

    let res = db_client
        .query(
            &select_req,
            &[
                &format!(
                    "%{}%",
                    state.filters.name.as_ref().unwrap_or(&String::new())
                ),
                &format!(
                    "%{}{}%",
                    match_start,
                    state.filters.city.as_ref().unwrap_or(&String::new())
                ),
                &format!(
                    "%{}%",
                    state.filters.vendor.as_ref().unwrap_or(&String::new())
                ),
                &(state.filters.min_price.unwrap_or_default() as f32),
                &(state.filters.max_price.unwrap_or_else(|| 10000) as f32),
                &(state.pagination.per_page as i64),
                &((state.pagination.page * state.pagination.per_page) as i64),
            ],
        )
        .await?;

    log::debug!(
        "Took {} after req",
        (chrono::Utc::now() - now).num_milliseconds()
    );
    let mut games = Games {
        ..Default::default()
    };
    for row in res {
        let game = match craft_game_from_row(db_client, row).await {
            Ok(game) => {
                //log::debug!("[DB] game crafted from DB: {:#?}", game);
                game
            }
            Err(e) => {
                log::error!("[DB] craft game from row error : {}", e);
                return Err(e);
            }
        };
        games.games.push(Box::new(game))
    }

    Ok(games)
}

pub async fn select_count_filtered_games_from_db(
    db_client: &Client,
    filters: Filters,
) -> Result<i64, Error> {
    let select_req = format!(
        "SELECT COUNT(*) FROM (
                SELECT oa.oa_id
                FROM okkazeo_announce oa
                JOIN deal d on d.deal_oa_id = oa.oa_id
                LEFT JOIN reviewer r on r.reviewer_oa_id = oa.oa_id
                JOIN seller s on s.seller_id = oa.oa_seller
                WHERE unaccent(oa.oa_name) ilike unaccent($1) AND unaccent(oa.oa_city) ilike unaccent($2)
                AND unaccent(s.seller_name) ilike unaccent($3)
                AND oa.oa_price > $4 AND oa.oa_price < $5
                {}
                GROUP BY oa.oa_id
                {}
        ) AS c;",
        if filters.pro.is_some() {
            "AND NOT s.seller_is_pro"
        } else {
            ""
        },
        if filters.note.is_some() {
            format!(
                "HAVING SUM(CASE WHEN r.reviewer_number > 0 THEN r.reviewer_note * r.reviewer_number ELSE 0 END) / SUM(CASE WHEN r.reviewer_number > 0 
                 THEN r.reviewer_number ELSE 1 END) > {}",
 //               "HAVING AVG(r.reviewer_note* r.reviewer_number) / SUM(r.reviewer_number) > {}",
                filters.note.unwrap()
            )
        } else {
            "".to_string()
        },
    );

    let mut match_start = "";
    if filters.city.is_some() && filters.city.as_ref().unwrap().parse::<i32>().is_ok() {
        match_start = "(";
    }

    let res = db_client
        .query(
            &select_req,
            &[
                &format!("%{}%", filters.name.unwrap_or_default()),
                &format!("%{}{}%", match_start, filters.city.unwrap_or_default()),
                &format!("%{}%", filters.vendor.unwrap_or_default()),
                &(filters.min_price.unwrap_or_default() as f32),
                &(filters.max_price.unwrap_or_else(|| 10000) as f32),
            ],
        )
        .await?;

    let nbr: i64 = res.get(0).unwrap().try_get(0)?;

    Ok(nbr)
}

pub async fn check_if_seller_in_db(db_client: &Client, id: i32) -> Result<i32, Error> {
    log::debug!("checkin if seller is in db");
    let select_req = format!(
        "SELECT seller_id
                FROM seller
                WHERE seller_id = $1"
    );

    let res = db_client.query(&select_req, &[&id]).await?;

    Ok(res.len() as i32)
}

pub async fn select_shipping_from_db(
    db_client: &Client,
    id: i32,
) -> Result<HashMap<String, f32>, Error> {
    let select_req = format!(
        "SELECT *
                FROM shipping
                WHERE ship_oa_id = $1"
    );

    let res = db_client.query(&select_req, &[&id]).await?;

    let mut ships = HashMap::<String, f32>::new();
    for row in res {
        let shipper = row.try_get("ship_shipper")?;
        let price = row.try_get("ship_price")?;
        ships.insert(shipper, price);
    }

    Ok(ships)
}

pub async fn select_intervalled_ids_from_oa_table_from_db(
    db_client: &Client,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<Vec<i32>, Error> {
    let select_req = format!(
        "SELECT oa_id
                FROM okkazeo_announce oa WHERE oa_last_modification_date > $1 AND oa_last_modification_date < $2"
    );

    let res = db_client
        .query(&select_req, &[&start_date, &end_date])
        .await?;

    res.into_iter().map(|row| row.try_get("oa_id")).collect()
}

pub async fn select_all_ids_from_oa_table_from_db(db_client: &Client) -> Result<Vec<i32>, Error> {
    let select_req = format!(
        "SELECT oa_id
                FROM okkazeo_announce"
    );

    let res = db_client.query(&select_req, &[]).await?;

    res.into_iter().map(|row| row.try_get("oa_id")).collect()
}

pub async fn select_references_from_db(
    db_client: &Client,
    id: i32,
) -> Result<HashMap<String, Reference>, Error> {
    let select_req = format!(
        "SELECT *
                FROM reference
                WHERE ref_oa_id = $1"
    );

    let res = db_client.query(&select_req, &[&id]).await?;

    let mut refs = HashMap::<String, Reference>::new();
    for row in res {
        let name: String = row.try_get("ref_name")?;
        let price = row.try_get("ref_price")?;
        let url = row.try_get("ref_url")?;
        refs.insert(name.clone(), Reference { name, price, url });
    }

    Ok(refs)
}

pub async fn select_reviews_from_db(db_client: &Client, id: i32) -> Result<Review, Error> {
    let select_req = format!(
        "SELECT *
                FROM reviewer
                WHERE reviewer_oa_id = $1"
    );

    let res = db_client.query(&select_req, &[&id]).await?;

    let mut revs = HashMap::<String, Reviewer>::new();
    for row in res {
        let name: String = row.try_get("reviewer_name")?;
        let url = row.try_get("reviewer_url")?;
        let note = row.try_get("reviewer_note")?;
        let number: i32 = row.try_get("reviewer_number")?;
        revs.insert(
            name.clone(),
            Reviewer {
                name,
                url,
                note,
                number: number as u32,
            },
        );
    }

    let mut rev = Review {
        reviews: revs,
        average_note: 0.0,
    };
    rev.compute_average_note();

    Ok(rev)
}
