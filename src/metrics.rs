use axum::{routing::get, Router};
use prometheus::{register_int_counter, IntCounter, TextEncoder, Encoder};
use lazy_static::lazy_static;

async fn metrics() -> String {
    AXUM_METRICS_GET.inc();
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    encoder
        .encode(&prometheus::gather(), &mut buffer)
        .expect("Failed to encode metrics");

    String::from_utf8(buffer).expect("Failed to convert bytes to string")
}

pub async fn run_metrics(bind_addr: String) {
    let app = Router::new().route("/metrics", get(metrics));

    log::info!("[METRICS] starting metrics server on {}", bind_addr);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

lazy_static! {
    static ref AXUM_METRICS_GET: IntCounter = register_int_counter!(
        "axum_metrics_get",
        "Number of get resquests to metrics route"
    )
    .unwrap();
}
