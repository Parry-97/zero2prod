use std::{net::TcpListener, task::Wake};

use sqlx::{Executor, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    startup::run,
};

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
    //WARN: when we call await on `run` it starts listening ,on the address we specified,
    //indefinetely. It never returns and our test logic never gets executed.

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.db_name = Uuid::new_v4().to_string();

    let pool = configure_database(&configuration.database).await;

    let server = run(listener, pool.clone()).expect("Failed to bind address");
    //NOTE: We need to use `tokio::spawn` to run it as a background task
    tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut pool = sqlx::PgPool::connect(config.connection_string_without_db().as_str())
        .await
        .expect("Failed to connect to Postgres");

    pool.execute(format!(r#"CREATE DATABASE "{}";"#, config.db_name).as_str())
        .await
        .expect("Failed to create DB");

    pool = PgPool::connect(config.connection_string().as_str())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate the database");

    pool
}
