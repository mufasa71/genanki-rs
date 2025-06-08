pub mod configuration;
pub mod routes;

use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use routes::{health_check, subscribe};
use sqlx::PgPool;

use crate::routes::get_subscription;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .service(subscribe)
            .service(get_subscription)
            .route("/health_check", web::get().to(health_check))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
