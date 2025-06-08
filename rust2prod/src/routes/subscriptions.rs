use actix_web::{HttpResponse, Responder, get, post, web};
use serde::Serialize;
use sqlx::{
    Error, FromRow,
    postgres::PgPool,
    types::{
        Uuid,
        chrono::{DateTime, Utc},
    },
};

use crate::configuration::get_configuration;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    user_name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Info {
    email: String,
}

#[derive(Debug, FromRow, Serialize)]
struct Subscription {
    id: Uuid,
    email: String,
    user_name: String,
    subscribed_at: DateTime<Utc>,
}

pub async fn get_pool() -> Result<sqlx::Pool<sqlx::Postgres>, Error> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_string = configuration.database.connection_string();

    PgPool::connect(&connection_string).await
}

#[post("/subscriptions")]
pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> HttpResponse {
    let rec = sqlx::query!(
        "INSERT INTO subscriptions ( email, user_name ) VALUES ( $1, $2 )",
        form.email,
        form.user_name
    )
    .execute(db_pool.as_ref())
    .await;

    if let Ok(_rec) = rec {
        return HttpResponse::Ok().finish();
    }

    HttpResponse::BadRequest().finish()
}

#[get("/subscriptions")]
pub async fn get_subscription(
    info: web::Query<Info>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let rec = sqlx::query_as::<_, Subscription>("SELECT * FROM subscriptions WHERE email = $1")
        .bind(&info.email)
        .fetch_one(db_pool.get_ref())
        .await;

    match rec {
        Ok(rec) => HttpResponse::Ok().json(web::Json(rec)),
        Err(_e) => HttpResponse::NotFound().finish(),
    }
}
