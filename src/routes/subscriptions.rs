use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<FormData>, app_state: web::Data<PgPool>) -> impl Responder {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );

    //WARNING: Calling enter inside an async block or function can cause the
    //current span to be lost.This is because the current span is stored
    //in thread-local storage, which is not accessible from async contexts.
    let _request_span_guard = request_span.enter();

    //NOTE: We don't need to call enter on query_span because instrument takes care of it automatically
    //in the query lifetime
    let query_span = tracing::info_span!("Saving new subscriber details in the database");
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
    .instrument(query_span)
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
