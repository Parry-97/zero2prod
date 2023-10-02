use actix_web::{
    web::{self, Form},
    HttpRequest, HttpResponse, Responder,
};
use sqlx::{Pool, Postgres};

use crate::FormData;

pub async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json("All good!")
}

pub async fn subscribe(db: web::Data<Pool<Postgres>>, form_data: Form<FormData>) -> HttpResponse {
    let request_id = uuid::Uuid::new_v4();
    log::info!(
        "request_id {} -- Adding '{}' '{} as a new subscriber'",
        request_id,
        form_data.email,
        form_data.name
    );
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3,$4)"#,
        uuid::Uuid::new_v4(),
        form_data.email,
        form_data.name,
        chrono::Utc::now(),
    )
    .execute(db.get_ref())
    .await
    {
        Ok(_) => {
            log::info!("request_id {} -- New subscriber saved.", request_id);
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            log::error!(
                "request_id {} -- Failed to execute query: {:?}",
                request_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
