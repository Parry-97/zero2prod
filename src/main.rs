use std::net;

use zero2prod::{
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let port = std::env::var("PORT").expect("Failed to get PORT env var");
    let addr = format!("localhost:{}", port);
    let listener = net::TcpListener::bind(addr)?;
    let connection_pool = zero2prod::startup::configure_database().await;
    run(listener, connection_pool)?.await
}
