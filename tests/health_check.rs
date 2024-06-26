use std::net::TcpListener;

use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Executor, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    email_client::EmailClient,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_level = "info";
    let subscriber_name = "test";

    //WARN: We cannot assign the output of `get_subscriber` to a variable based on value of `TEST_LOG`
    //to avoid repetitions because the sink is part of the actual concrete type returned by
    //`get_subscriber` (something like `Layered<...,Sink,>`), therefore they are not the same type.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(
            subscriber_name.into(),
            default_level.into(),
            std::io::stdout,
        );
        init_subscriber(subscriber);
    } else {
        let subscriber =
            get_subscriber(subscriber_name.into(), default_level.into(), std::io::sink);
        init_subscriber(subscriber);
    }
});

#[derive(Debug)]
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

//NOTE: `tokio::test` is the testing equivalent of `tokio::main`.
//You can inspect the generated code using `cargo expand --test health_check (<- name of the file)`
#[tokio::test]
async fn health_check_works() {
    let client = reqwest::Client::new();
    let test_app = spawn_app().await;

    let response = client
        .get(&format!("{}/health_check", test_app.address))
        .send()
        .await
        .expect("Failed to execute request. ");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request. ");

    assert_eq!(200, response.status().as_u16());

    let pool = test_app.db_pool;
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let client = reqwest::Client::new();
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("name=&email=ursula_le_guin%40gmail.com", "name is empty"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request. ");

        assert_eq!(400,response.status().as_u16(),
        "The API did not fail but returned 400 Bad Request when the payload was {error_message}");
    }
}

//NOTE: This function is the only piece in our tests that depends on the application code.
//Everything else is decoupled from the underlying implementation details
async fn spawn_app() -> TestApp {
    //The first time `initialize` is invoked the code in `TRACING` is executed.
    //All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.db_name = Uuid::new_v4().to_string();

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");

    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
    );
    let pool = configure_database(&configuration.database).await;

    let server = run(listener, pool.clone(), email_client).expect("Failed to bind address");
    //NOTE: We need to use `tokio::spawn` to run it as a background task
    tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut pool = sqlx::PgPool::connect(
        config
            .connection_string_without_db()
            .expose_secret()
            .as_str(),
    )
    .await
    .expect("Failed to connect to Postgres");

    pool.execute(format!(r#"CREATE DATABASE "{}";"#, config.db_name).as_str())
        .await
        .expect("Failed to create DB");

    pool = PgPool::connect(config.connection_string().expose_secret().as_str())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate the database");

    pool
}
