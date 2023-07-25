use crate::{handlers::handler::ErrorResponse, repositories::user_repository::UserRepository};
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;

use super::handler::DataResponse;

async fn get_user_handler(pool: web::Data<PgPool>, path: web::Path<String>) -> impl Responder {
    let user = UserRepository::new(&pool).get_one(&path).await;
    match user {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::NotFound().finish(),
    }
}

async fn get_all_users_handler(pool: web::Data<PgPool>) -> impl Responder {
    let users = UserRepository::new(&pool).get_all().await;
    HttpResponse::Ok().json(DataResponse::new(users))
}

#[derive(serde::Deserialize)]
struct CreateUserRequest {
    pub npub: String,
}

async fn create_user_handler(
    pool: web::Data<PgPool>,
    user: web::Json<CreateUserRequest>,
) -> impl Responder {
    match UserRepository::new(&pool).create(&user.npub).await {
        Ok(created_user) => HttpResponse::Created().json(DataResponse::new(created_user)),
        Err(_) => {
            HttpResponse::BadRequest().json(ErrorResponse::new("Npub already exists".to_string()))
        }
    }
}

async fn delete_user_handler(pool: web::Data<PgPool>, path: web::Path<String>) -> impl Responder {
    let user_npub = path.into_inner();
    match UserRepository::new(&pool).delete(&user_npub).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/users/{user_npub}")
            .route(web::get().to(get_user_handler))
            .route(web::delete().to(delete_user_handler)),
    )
    .route("/users", web::get().to(get_all_users_handler))
    .route("/users", web::post().to(create_user_handler));
}
