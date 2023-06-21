use actix_web::{web, HttpResponse, ResponseError};
use sqlx::PgPool;
use crate::routes::error_chain_fmt;
use actix_web::http::StatusCode;

struct ConfirmedSubscriber {
    email: String,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<ConfirmedSubscriber>, anyhow::Error> {
    let rows = sqlx::query_as!(
        ConfirmedSubscriber,
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}
#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}
pub async fn publish_newsletter(
    _body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let _confirmed_subscribers = get_confirmed_subscribers(&pool).await?;
    HttpResponse::Ok().finish()
}
