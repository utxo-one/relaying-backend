use crate::models::user::User;
use chrono::NaiveDateTime;
use sqlx::postgres::PgPool;

pub struct UserRepository<'a> {
    pub pool: &'a PgPool,
}

impl<'a> UserRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_one(&self, user_npub: &str) -> Option<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE npub = $1")
            .bind(user_npub)
            .fetch_optional(self.pool)
            .await;

        match user {
            Ok(Some(user)) => Some(User::from_db_user(user)),
            _ => None,
        }
    }

    pub async fn get_all(&self) -> Vec<User> {
        let db_users = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(self.pool)
            .await
            .unwrap();

        db_users.into_iter().map(User::from_db_user).collect()
    }

    pub async fn create(&self, user_npub: &str) -> Result<User, sqlx::Error> {
        let db_user: User =
            sqlx::query_as::<_, User>("INSERT INTO users (npub) VALUES ($1) RETURNING *")
                .bind(user_npub)
                .fetch_one(self.pool)
                .await?;

        Ok(User::from_db_user(db_user))
    }

    pub async fn delete(&self, user_npub: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET deleted_at = $1 WHERE npub = $2")
            .bind(NaiveDateTime::from_timestamp_opt(0, 0))
            .bind(user_npub)
            .execute(self.pool)
            .await?;

        Ok(())
    }

    pub async fn user_exists(&self, user_npub: String) -> bool {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE npub = $1")
            .bind(user_npub)
            .fetch_optional(self.pool)
            .await;

        match user {
            Ok(Some(_)) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::util::generators::generate_random_string;

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

        let created_user = UserRepository::new(&pool)
            .create(user_npub.as_str())
            .await
            .expect("Failed to create user");
        assert_eq!(created_user.npub, user_npub);

        let retrieved_user = UserRepository::new(&pool)
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
        let repo = UserRepository::new(&pool);
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

        let created_user = UserRepository::new(&pool)
            .create(user_npub.as_str())
            .await
            .expect("Failed to create user");

        UserRepository::new(&pool)
            .delete(user_npub.as_str())
            .await
            .expect("Failed to delete user");

        let retrieved_user = UserRepository::new(&pool)
            .get_one(user_npub.as_str())
            .await
            .expect("Failed to retrieve user");
        assert!(retrieved_user.deleted_at.is_some());
    }
}
