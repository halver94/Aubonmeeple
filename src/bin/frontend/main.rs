use tokio::join;
use boardgame_finder::frontlib::server;
use boardgame_finder::metrics;

#[tokio::main]
async fn main() {
    log_panics::init();
    env_logger::init();

    let frontend_bind_addr = std::env::var("FRONTEND_ADDR").unwrap_or("0.0.0.0:3001".to_string());
    let frontend_metrics_bind_addr = std::env::var("FRONTEND_METRICS_ADDR").unwrap_or("127.0.0.1:3002".to_string());

    log::info!("[MAIN] starting program");
    join!(
        server::run_server(frontend_bind_addr),
        metrics::run_metrics(frontend_metrics_bind_addr),
    );
}
