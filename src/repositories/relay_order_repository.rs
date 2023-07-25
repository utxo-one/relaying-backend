use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;
use sqlx::PgPool;
use std::fmt;

use crate::models::relay_orders::{CreateRelayOrder, RelayOrder};

#[derive(Debug)]
pub enum RelayOrderRepositoryError {
    SqlxError(SqlxError),
    NotFound,
}

impl From<SqlxError> for RelayOrderRepositoryError {
    fn from(err: SqlxError) -> Self {
        RelayOrderRepositoryError::SqlxError(err)
    }
}

impl fmt::Display for RelayOrderRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RelayOrderRepositoryError::SqlxError(err) => err.fmt(f),
            RelayOrderRepositoryError::NotFound => write!(f, "Relay order not found"),
        }
    }
}

pub struct RelayOrderRepository<'a> {
    pub pool: &'a PgPool,
}

impl<'a> RelayOrderRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        relay_order: CreateRelayOrder,
    ) -> Result<RelayOrder, RelayOrderRepositoryError> {
        let uuid = uuid::Uuid::new_v4().to_string();
        let relay_order: RelayOrder = sqlx::query_as::<_, RelayOrder>(
            "
            INSERT INTO relay_orders (uuid, user_npub, amount, cloud_provider, instance_type, implementation, hostname, status)
            VALUES ($1, $2, $3, $4::relay_cloud_provider, $5::relay_instance_type, $6::relay_implementation, $7, $8::relay_order_status)
            RETURNING uuid, user_npub, amount, cloud_provider, instance_type, implementation, hostname, status, created_at, updated_at
            ")
            .bind(uuid)
            .bind(relay_order.user_npub)
            .bind(relay_order.amount)
            .bind(relay_order.cloud_provider.as_str())
            .bind(relay_order.instance_type.as_str())
            .bind(relay_order.implementation.as_str())
            .bind(relay_order.hostname)
            .bind(relay_order.status)
            .fetch_one(self.pool)
            .await?;

        Ok(relay_order)
    }

    pub async fn get_one(&self, uuid: String) -> Result<RelayOrder, RelayOrderRepositoryError> {
        let relay_order: RelayOrder = sqlx::query_as::<_, RelayOrder>(
            "
            SELECT uuid, user_npub, amount, cloud_provider, instance_type, implementation, hostname, status, created_at, updated_at
            FROM relay_orders
            WHERE uuid = $1
            ")
            .bind(uuid)
            .fetch_one(self.pool)
            .await?;

        Ok(relay_order)
    }

    pub async fn delete(&self, uuid: String) -> Result<(), RelayOrderRepositoryError> {
        sqlx::query(
            "
            DELETE FROM relay_orders
            WHERE uuid = $1
            ",
        )
        .bind(uuid)
        .execute(self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        models::{
            cloud_provider::{CloudProvider, InstanceType},
            relay::RelayImplementation,
            relay_orders::RelayOrderStatus,
        },
        repositories::user_repository::UserRepository,
        util::generators::generate_random_string,
    };

    use super::*;

    async fn create_test_pool() -> PgPool {
        dotenvy::dotenv().ok();

        let db_url =
            dotenvy::var("DATABASE_URL").expect("TEST_DATABASE_URL must be set to run tests");
        let pool = PgPool::connect(&db_url)
            .await
            .expect("Failed to create test pool");
        pool
    }

    async fn create_test_user(pool: &PgPool) -> String {
        let user_npub = generate_random_string(16).await;

        let user = UserRepository::new(&pool)
            .create(&user_npub)
            .await
            .expect("Failed to create user");

        user.npub
    }

    async fn delete_test_relay_order(pool: &PgPool, uuid: String) {
        RelayOrderRepository::new(&pool)
            .delete(uuid)
            .await
            .expect("Failed to delete relay order");
    }

    async fn delete_test_user(pool: &PgPool, npub: String) {
        UserRepository::new(&pool)
            .delete(&npub)
            .await
            .expect("Failed to delete user");
    }

    #[tokio::test]
    async fn test_create_and_delete_relay_order() {
        let pool = create_test_pool().await;
        let npub = create_test_user(&pool).await;

        let create = CreateRelayOrder {
            user_npub: npub.clone(),
            amount: 1,
            cloud_provider: CloudProvider::AWS,
            instance_type: InstanceType::AwsT2Nano,
            implementation: RelayImplementation::Strfry,
            hostname: "test".to_string(),
            status: RelayOrderStatus::Pending,
        };

        let repo = RelayOrderRepository::new(&pool);
        let relay_order = repo
            .create(create)
            .await
            .expect("Failed to create relay order");

        assert_eq!(relay_order.user_npub, npub);

        delete_test_user(&pool, npub).await;
        delete_test_relay_order(&pool, relay_order.uuid.clone()).await;

        repo.get_one(relay_order.uuid)
            .await
            .expect_err("Failed to delete relay order");
    }

    #[tokio::test]
    async fn test_get_relay_order() {
        let pool = create_test_pool().await;
        let npub = create_test_user(&pool).await;

        let create = CreateRelayOrder {
            user_npub: npub.clone(),
            amount: 1,
            cloud_provider: CloudProvider::AWS,
            instance_type: InstanceType::AwsT2Nano,
            implementation: RelayImplementation::Strfry,
            hostname: "test".to_string(),
            status: RelayOrderStatus::Pending,
        };

        let repo = RelayOrderRepository::new(&pool);

        let relay_order = repo
            .create(create)
            .await
            .expect("Failed to create relay order");

        let relay_order = repo
            .get_one(relay_order.uuid)
            .await
            .expect("Failed to get relay order");

        assert_eq!(relay_order.user_npub, npub);

        delete_test_user(&pool, npub).await;
        delete_test_relay_order(&pool, relay_order.uuid).await;
    }
}
