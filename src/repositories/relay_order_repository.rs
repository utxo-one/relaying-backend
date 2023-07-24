use sqlx::PgPool;
use crate::models::relay_orders::{RelayOrder, RelayOrderStatus};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateRelayOrder {
    pub user_npub: String,
    pub amount: i32,
    pub cloud_provider: String,
    pub instance_type: String,
    pub implementation: String,
    pub hostname: String,
    pub status: String,
}

pub async fn create_relay_order(
    relay_order: CreateRelayOrder,
    pool: &PgPool,
) -> Result<RelayOrder, sqlx::Error> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let relay_order: RelayOrder = sqlx::query_as::<_, RelayOrder>(
        "
        INSERT INTO relay_orders (uuid, user_npub, amount, cloud_provider, instance_type, implementation, hostname, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING uuid, user_npub, amount, cloud_provider, instance_type, implementation, hostname, status, created_at, updated_at
        ")
        .bind(uuid)
        .bind(relay_order.user_npub)
        .bind(relay_order.amount)
        .bind(relay_order.cloud_provider)
        .bind(relay_order.instance_type)
        .bind(relay_order.implementation)
        .bind(relay_order.hostname)
        .bind(relay_order.status)
    .fetch_one(pool)
    .await?;

    Ok(relay_order)
}

pub async fn get_relay_order(pool: &PgPool, uuid: String) -> Result<RelayOrder, sqlx::Error> {
    let relay_order: RelayOrder = sqlx::query_as::<_, RelayOrder>(
        "
        SELECT uuid, user_npub, amount, cloud_provider, instance_type, implementation, hostname, status, created_at, updated_at
        FROM relay_orders
        WHERE uuid = $1
        ")
        .bind(uuid)
        .fetch_one(pool)
        .await?;

    Ok(relay_order)
}

pub async fn delete_relay_order(pool: &PgPool, uuid: String) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        DELETE FROM relay_orders
        WHERE uuid = $1
        ",
    )
    .bind(uuid)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        models::relay_orders::RelayOrderStatus,
        repositories::user_repository::{create_user, delete_user},
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

        let user = create_user(&pool, &user_npub)
            .await
            .expect("Failed to create user");

        user.npub
    }

    async fn delete_test_relay_order(pool: &PgPool, uuid: String) {
        delete_relay_order(&pool, uuid)
            .await
            .expect("Failed to delete relay order");
    }

    async fn delete_test_user(pool: &PgPool, npub: String) {
        delete_user(&pool, &npub)
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
            cloud_provider: "test".to_string(),
            instance_type: "test".to_string(),
            implementation: "test".to_string(),
            hostname: "test".to_string(),
            status: RelayOrderStatus::Pending.to_string(),
        };

        let relay_order = create_relay_order(create, &pool)
            .await
            .expect("Failed to create relay order");

        assert_eq!(relay_order.user_npub, npub);

        delete_test_user(&pool, npub).await;
        delete_test_relay_order(&pool, relay_order.uuid.clone()).await;

        get_relay_order(&pool, relay_order.uuid)
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
            cloud_provider: "test".to_string(),
            instance_type: "test".to_string(),
            implementation: "test".to_string(),
            hostname: "test".to_string(),
            status: RelayOrderStatus::Pending.to_string(),
        };

        let relay_order = create_relay_order(create, &pool)
            .await
            .expect("Failed to create relay order");

        let relay_order = get_relay_order(&pool, relay_order.uuid)
            .await
            .expect("Failed to get relay order");

        assert_eq!(relay_order.user_npub, npub);

        delete_test_user(&pool, npub).await;
        delete_test_relay_order(&pool, relay_order.uuid).await;
    }
}
