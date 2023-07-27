use actix_web::{web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use nostr::prelude::FromBech32;
use secp256k1::XOnlyPublicKey;
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use serde::{Deserialize, Serialize};

use crate::util::{DataResponse, ErrorResponse, bech32_encode};

// -----------------------------------------------------------------------------
// Models & DTOs
// -----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub npub: String,
    pub hexpub: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl User {
    pub fn from_db_user(db_user: User) -> Self {
        User {
            npub: db_user.npub,
            hexpub: db_user.hexpub,
            created_at: db_user.created_at,
            updated_at: db_user.updated_at,
            deleted_at: db_user.deleted_at,
        }
    }
}

#[derive(serde::Deserialize)]
struct CreateUserDto {
    pub hexpub: String,
}

// -----------------------------------------------------------------------------
// Repository
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct UserRepository {
    pub pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_one(&self, user_npub: &str) -> Option<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE npub = $1")
            .bind(user_npub)
            .fetch_optional(&self.pool)
            .await;

        match user {
            Ok(Some(user)) => Some(User::from_db_user(user)),
            _ => None,
        }
    }

    pub async fn get_all(&self) -> Vec<User> {
        let db_users = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&self.pool)
            .await
            .unwrap();

        db_users.into_iter().map(User::from_db_user).collect()
    }

    pub async fn create(&self, npub: &str, hexpub: &str) -> Result<User, sqlx::Error> {
        let db_user: User =
            sqlx::query_as::<_, User>("INSERT INTO users (npub, hexpub) VALUES ($1, $2) RETURNING *")
                .bind(npub)
                .bind(hexpub)
                .fetch_one(&self.pool)
                .await?;

        Ok(User::from_db_user(db_user))
    }

    pub async fn delete(&self, user_npub: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET deleted_at = $1 WHERE npub = $2")
            .bind(NaiveDateTime::from_timestamp_opt(0, 0))
            .bind(user_npub)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn user_exists(&self, user_npub: String) -> bool {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE npub = $1")
            .bind(user_npub)
            .fetch_optional(&self.pool)
            .await;

        match user {
            Ok(Some(_)) => true,
            _ => false,
        }
    }
}

// -----------------------------------------------------------------------------
// Handlers
// -----------------------------------------------------------------------------

async fn get_user_handler(user_repo: web::Data<UserRepository>, path: web::Path<String>) -> impl Responder {
    let user = user_repo.get_one(&path).await;
    match user {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::NotFound().finish(),
    }
}

async fn get_all_users_handler(user_repo: web::Data<UserRepository>) -> impl Responder {
    let users = user_repo.get_all().await;
    HttpResponse::Ok().json(DataResponse::new(users))
}

async fn create_user_handler(
    user_repo: web::Data<UserRepository>,
    user: web::Json<CreateUserDto>,
) -> impl Responder {

    let user_npub = bech32_encode(&user.hexpub);

    eprintln!("user_npub: {:?}", user_npub);

    match user_npub {
        Ok(user_npub) => {
            match user_repo.create(&user_npub, &user.hexpub).await {
                Ok(created_user) => HttpResponse::Created().json(DataResponse::new(created_user)),
                Err(e) => {
                    HttpResponse::BadRequest().json(ErrorResponse::new(e.to_string()))
                }
            }
        }
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse::new("Invalid hex pub key".to_string()));
        }
    }
}

async fn delete_user_handler(user_repo: web::Data<UserRepository>, path: web::Path<String>) -> impl Responder {
    let user_npub = path.into_inner();
    match user_repo.delete(&user_npub).await {
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

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------


#[cfg(test)]
mod tests {

    use crate::util::{generate_random_string, TestUtils};

    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_create_and_get_user() {

        let test_utils = TestUtils::new().await;
        let user = test_utils.create_user().await;

    }
}
