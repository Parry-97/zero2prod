use std::net::TcpListener;

use actix_web::{HttpRequest, Responder};
use secrecy::ExposeSecret;
use zero2prod::{
    configuration::get_configuration,
    email_client::EmailClient,
    telemetry::{get_subscriber, init_subscriber},
};

async fn _greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", name)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");

    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
    );
    //NOTE: The Server must be awaited and polled to start running. It resolves when it is shuts down
    let listener = TcpListener::bind(address)?;
    let pool = sqlx::PgPool::connect_lazy(
        configuration
            .database
            .connection_string()
            .expose_secret()
            .as_str(),
    )
    // .await
    .expect("Failed to connect to Postgres");
    zero2prod::startup::run(listener, pool, email_client)?.await
}
