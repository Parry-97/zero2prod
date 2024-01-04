use std::net::TcpListener;

use actix_web::{HttpRequest, Responder};
use zero2prod::{
    configuration::get_configuration,
    telemetry::{get_subscriber, init_subscriber},
};

async fn _greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", name)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into());
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    let address = format!("localhost:{}", configuration.app_port);
    //NOTE: The Server must be awaited and polled to start running. It resolves when it is shuts down
    let listener = TcpListener::bind(address)?;
    let pool = sqlx::PgPool::connect(configuration.database.connection_string().as_str())
        .await
        .expect("Failed to connect to Postgres");
    zero2prod::startup::run(listener, pool)?.await
}
