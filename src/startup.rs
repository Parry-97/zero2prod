use std::net::TcpListener;

use actix_web::{dev::Server, middleware::Logger, web, App, HttpServer};
use env_logger::Env;
use sqlx::PgPool;

use crate::routes::{health_check, subscribe};

pub fn run(listener: TcpListener, pool: PgPool) -> Result<Server, std::io::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            //NOTE: Register the connection pool as part of the application state
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
