use std::net::TcpListener;

use actix_web::{
    dev::Server,
    middleware::Logger,
    web::{self},
    App, HttpServer,
};
use dotenvy::dotenv;
use routes::{greet, health_check, subscribe};
use serde::Deserialize;
use sqlx::PgPool;

mod routes;

#[derive(Debug, Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn configure_test_database() -> PgPool {
    dotenv().ok();
    let connection_string = std::env::var("POSTGRES_URL").expect("POSTGRES_URL must be set.");
    let test_db_name = uuid::Uuid::new_v4().to_string();
    let connection_string = connection_string.replace("newsletter", &test_db_name);
    let pool = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::query(&*format!(r#"CREATE DATABASE "{}";"#, test_db_name))
        .execute(&pool)
        .await
        .expect("Failed to create test database.");

    let connection_string = connection_string.replace("newsletter", &test_db_name);
    let pool = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to database.");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate database.");

    pool
}

pub async fn configure_database() -> sqlx::PgPool {
    dotenv().ok();
    let connection_string = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    sqlx::PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.")
}

pub fn run(listener: TcpListener, pool: PgPool) -> Result<Server, std::io::Error> {
    let pool = web::Data::new(pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/", web::get().to(greet))
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
