use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};

use crate::game::Games;

async fn handle_request(
    _req: Request<Body>,
    arc_games: Arc<Mutex<Games>>,
) -> Result<Response<Body>, Infallible> {
    let games = arc_games.lock().unwrap().clone();
    let response_body = games.create_html_table();

    let response = Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=UTF-8")
        .body(Body::from(response_body))
        .unwrap();

    Ok(response)
}

pub async fn set_server(games: &mut Arc<std::sync::Mutex<Games>>) {
    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(|_conn| {
        let games = games.clone();
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async { Ok::<_, Infallible>(service_fn(move |req| handle_request(req, games.clone()))) }
    });

    let addr = ([0, 0, 0, 0], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
