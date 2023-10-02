use std::net;

use env_logger::Env;
use zero2prod::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    //NOTE: // `init` does call `set_logger`, so this is all we need to do.
    // We are falling back to printing all logs at info-level or above
    // if the RUST_LOG environment variable has not been set.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let port = std::env::var("PORT").expect("Failed to get PORT env var");
    let addr = format!("localhost:{}", port);
    let listener = net::TcpListener::bind(addr)?;
    let connection_pool = zero2prod::configure_database().await;
    run(listener, connection_pool)?.await
}
