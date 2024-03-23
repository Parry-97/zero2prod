use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberName};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, app_state),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, app_state: web::Data<PgPool>) -> impl Responder {
    let name = match SubscriberName::parse(form.0.name) {
        Err(_) => return HttpResponse::BadRequest().finish(),
        Ok(name) => name,
    };
    // `web::Form` is a tuple struct around `FormData`
    // `form.0` gives us access to the underlying `FormData`
    // or we can use the `into_inner` method as well
    let new_subscriber = NewSubscriber {
        name,
        email: form.0.email,
    };
    match insert_subscriber(app_state.get_ref(), &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub fn is_valid_name(s: &str) -> bool {
    let is_empty_or_whitespace = s.trim().is_empty();

    //NOTE: A grapheme is defined by the Unicode standard as a "user-perceived" character: an
    //example would be an Umlaut letter, but it is composed of two characters
    //`graphemes` returns an iterator over the graphemes in the input `s`. `true` specifies that we
    //want to use the extended grapheme definition set, the recommended one
    let is_too_long = s.graphemes(true).count() > 256;

    let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_chars = s.chars().any(|g| forbidden_chars.contains(&g));

    !(is_empty_or_whitespace || is_too_long || contains_forbidden_chars)
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email,name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        new_subscriber.email,
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
