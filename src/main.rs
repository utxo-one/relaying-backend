use actix_cors::Cors;
use actix_web::{web::Data, App, HttpServer};
use sqlx::postgres::PgPool;
use std::{env, sync::Arc};

mod auth;
mod aws;
mod cloud_provider;
mod middleware;
mod relay;
mod relay_order;
mod user;
mod util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL not found in environment variables.");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create PostgreSQL pool.");

    let user_repo = user::UserRepository::new(pool.clone());
    let relay_order_repo = relay_order::RelayOrderRepository::new(pool.clone());
    let relay_repo = relay::RelayRepository::new(pool.clone());

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(user_repo.clone()))
            .app_data(Data::new(relay_order_repo.clone()))
            .app_data(Data::new(relay_repo.clone()))
            .configure(user::configure_routes)
            .configure(auth::configure_routes)
            .configure(relay_order::configure_routes) // Mount the user handlers
    })
    .bind("127.0.0.1:8888")?
    .run()
    .await?;

    Ok(())
}
