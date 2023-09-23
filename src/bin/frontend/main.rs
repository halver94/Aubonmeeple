use std::env;

use boardgame_finder::frontlib::server;
use log::Level;

#[tokio::main]
async fn main() {
    log_panics::init();

    //this one is for vscode
    env::set_var("RUST_LOG", "boardgame_finder=debug");
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(Level::Debug.as_str()),
    )
    .init();

    log::info!("[MAIN] starting program");
    let join = tokio::spawn(async move { server::set_server().await });
    let _ = join.await;
}
