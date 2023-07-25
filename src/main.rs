use actix_cors::Cors;
use actix_web::{http, web::Data, App, HttpServer};
use middleware::cors_middleware::{self, cors_middleware};
use sqlx::postgres::PgPool;
use std::env;

mod handlers {
    pub mod auth_handler;
    pub mod handler;
    pub mod relay_order_handler;
    pub mod user_handler;
}

mod models {
    pub mod cloud_provider;
    pub mod jwt;
    pub mod nostr;
    pub mod relay;
    pub mod relay_orders;
    pub mod user;
}

mod repositories {
    pub mod relay_order_repository;
    pub mod relay_repository;
    pub mod user_repository;
}

mod services {
    pub mod aws_service;
    pub mod jwt_service;
    pub mod nostr_service;
    pub mod relay_service;
}

mod middleware {
    pub mod cors_middleware;
    pub mod jwt_middleware;
}

mod util {
    pub mod generators;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();
    // Load the DATABASE_URL environment variable
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL not found in environment variables.");

    // Create the PostgreSQL pool
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create PostgreSQL pool.");

    // Start the Actix Web server
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(Data::new(pool.clone())) // Share the pool across all routes
            .configure(handlers::user_handler::configure_routes)
            .configure(handlers::auth_handler::configure_routes)
            .configure(handlers::relay_order_handler::configure_routes) // Mount the user handlers
    })
    .bind("127.0.0.1:8888")?
    .run()
    .await;

    Ok(())
}
