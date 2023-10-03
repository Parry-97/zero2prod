use routes::health_check;

use actix_web::{dev::Server, middleware::Logger, web, App, HttpServer};

use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use std::net::TcpListener;

use dotenvy::dotenv;

use crate::routes::{self, subscriptions};

pub async fn configure_test_database() -> PgPool {
    dotenv().ok();
    let connection_string = std::env::var("POSTGRES_URL").expect("POSTGRES_URL must be set.");
    let test_db_name = uuid::Uuid::new_v4().to_string();
    let pool = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::query(&*format!(r#"CREATE DATABASE "{}";"#, test_db_name))
        .execute(&pool)
        .await
        .expect("Failed to create test database.");

    let connection_string = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
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
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check::health_check))
            .route("/subscriptions", web::post().to(subscriptions::subscribe))
            .app_data(pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
