use actix_web::{web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use sqlx::FromRow;

use crate::util::{DataResponse, ErrorResponse};

// -----------------------------------------------------------------------------
// Models & DTOs
// -----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub npub: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl User {
    pub fn from_db_user(db_user: User) -> Self {
        User {
            npub: db_user.npub,
            created_at: db_user.created_at,
            updated_at: db_user.updated_at,
            deleted_at: db_user.deleted_at,
        }
    }
}

#[derive(serde::Deserialize)]
struct CreateUserDto {
    pub npub: String,
}

// -----------------------------------------------------------------------------
// Repository
// -----------------------------------------------------------------------------

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

    pub async fn create(&self, user_npub: &str) -> Result<User, sqlx::Error> {
        let db_user: User =
            sqlx::query_as::<_, User>("INSERT INTO users (npub) VALUES ($1) RETURNING *")
                .bind(user_npub)
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

async fn get_user_handler(
    user_repo: web::Data<UserRepository>,
    path: web::Path<String>,
) -> impl Responder {
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
    match user_repo.create(&user.npub).await {
        Ok(created_user) => HttpResponse::Created().json(DataResponse::new(created_user)),
        Err(_) => {
            HttpResponse::BadRequest().json(ErrorResponse::new("Npub already exists".to_string()))
        }
    }
}

async fn delete_user_handler(
    user_repo: web::Data<UserRepository>,
    path: web::Path<String>,
) -> impl Responder {
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

    use crate::util::generate_random_string;

    use super::*;
    use std::env;

    async fn create_test_pool() -> PgPool {
        dotenvy::dotenv().ok();

        let db_url =
            env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set to run tests");
        let pool = PgPool::connect(&db_url)
            .await
            .expect("Failed to create test pool");

        clean_up_data(&pool).await;
        pool
    }

    async fn clean_up_data(pool: &PgPool) {
        sqlx::query!("DELETE FROM users WHERE npub != null")
            .execute(pool)
            .await
            .expect("Failed to clean up data");
    }

    #[tokio::test]
    async fn test_create_and_get_user() {
        let pool = create_test_pool().await;
        let user_npub = generate_random_string(16).await;

        let created_user = UserRepository::new(pool.clone())
            .create(user_npub.as_str())
            .await
            .expect("Failed to create user");
        assert_eq!(created_user.npub, user_npub);

        let retrieved_user = UserRepository::new(pool.clone())
            .get_one(user_npub.as_str())
            .await
            .expect("Failed to retrieve user");
        assert_eq!(retrieved_user.npub, user_npub);
    }

    #[tokio::test]
    async fn test_get_all_users() {
        let pool = create_test_pool().await;
        clean_up_data(&pool).await;

        let user_npub = generate_random_string(16).await;
        let repo = UserRepository::new(pool);
        let created_user = repo
            .create(user_npub.as_str())
            .await
            .expect("Failed to create user");

        let all_users = repo.get_all();

        // print all users
        for user in all_users.await {
            assert!(user.npub.len() > 0);
        }
    }

    #[tokio::test]
    async fn test_delete_user() {
        let pool = create_test_pool().await;
        let user_npub = generate_random_string(16).await;

        clean_up_data(&pool).await;

        let created_user = UserRepository::new(pool.clone())
            .create(user_npub.as_str())
            .await
            .expect("Failed to create user");

        UserRepository::new(pool.clone())
            .delete(user_npub.as_str())
            .await
            .expect("Failed to delete user");

        let retrieved_user = UserRepository::new(pool.clone())
            .get_one(user_npub.as_str())
            .await
            .expect("Failed to retrieve user");
        assert!(retrieved_user.deleted_at.is_some());
    }
}
