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
    let request = Uuid::new_v4();
    log::info!(
        "request_id {} - Adding '{}' `{}` as a new subscriber",
        request,
        form.email,
        form.name
    );
    log::info!("Saving new subscribers details in the database");
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
        Ok(_) => {
            log::info!("New subscriber details have been added");
            HttpResponse::Ok()
        }
        Err(e) => {
            log::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError()
        }
    }
}
