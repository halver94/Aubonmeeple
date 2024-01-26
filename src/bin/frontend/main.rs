use boardgame_finder::frontlib::server;

#[tokio::main]
async fn main() {
    log_panics::init();
    env_logger::init();

    log::info!("[MAIN] starting program");
    let join = tokio::spawn(async move { server::set_server().await });
    let _ = join.await;
}
