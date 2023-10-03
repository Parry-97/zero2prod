use actix_web::{
    web::{self, Form},
    HttpResponse,
};
use serde::Deserialize;
use sqlx::{self, Pool, Postgres};

#[derive(Debug, Deserialize)]
pub struct FormData {
    pub(crate) email: String,
    pub(crate) name: String,
}

/// INFO: `#[tracing::instrument]` creates a span at the beginning of the function invocation and automatically
/// attaches all arguments passed to the function to the context of the span - in our case, form and pool.
/// Often function arguments won’t be displayable on log records (e.g. pool) or we’d like to
/// specify more explicitly what should/how they should be captured (e.g. naming each field of form) -
/// we can explicitly tell tracing to ignore them using the skip directive.
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form_data, db),
    fields(
        subscriber_email = %form_data.email,
        subscriber_name = %form_data.name
    )
)]
pub async fn subscribe(db: web::Data<Pool<Postgres>>, form_data: Form<FormData>) -> HttpResponse {
    match insert_subscriber(&db, &form_data).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form_data, pool)
)]
pub async fn insert_subscriber(
    pool: &Pool<Postgres>,
    form_data: &FormData,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (name, email)
        VALUES ($1, $2)
        "#,
        form_data.name,
        form_data.email
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
