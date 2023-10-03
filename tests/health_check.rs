use std::net::TcpListener;

use dotenvy::dotenv;
use once_cell::sync::Lazy;
use sqlx::PgPool;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

//NOTE: This is an example of a lazy static variable. It is initialised the first time it is accessed.
// This is useful for expensive initialisation procedures, such as setting up a tracing subscriber.
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

#[tokio::test]
async fn health_check_works() {
    let pool = zero2prod::startup::configure_test_database().await;
    let app_address = spawn_app(pool);
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", &app_address))
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(response.status().is_success());
    assert_eq!(Some(11), response.content_length());
}

fn spawn_app(pool: PgPool) -> String {
    dotenv().ok();
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("localhost:0").expect("Failed to bind random port.");
    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::startup::run(listener, pool).expect("Failed to bind address.");
    tokio::spawn(server);
    format!("http://localhost:{}", port)
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let pool = zero2prod::startup::configure_test_database().await;
    let app_address = spawn_app(pool);
    let client = reqwest::Client::new();
    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn check_db_connection() {
    dotenv().ok();
    let connection_string = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let _ = sqlx::PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");
}

#[tokio::test]
//NOTE: This is an example of table-driven test also known as parametrised test.
// It is particularly helpful when dealing with bad inputs - instead of duplicating test logic several
// times we can simply run the same assertion against a collection of known invalid bodies that we
// expect to fail in the same way.
async fn subscribe_returns_a_400_when_data_is_missing() {
    let pool = zero2prod::startup::configure_test_database().await;
    let app_address = spawn_app(pool);
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}
