use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<FormData>, app_state: web::Data<PgPool>) -> impl Responder {
    match sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email,name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(app_state.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError()
        }
    }
}
