use crate::models::relay::Relay;
use chrono::NaiveDateTime;
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::types::Json;
use sqlx::Error;
use uuid::Uuid;

pub async fn get_relay(pool: &PgPool, uuid: String) -> Option<Relay> {
    let relay = sqlx::query_as::<_, Relay>("SELECT * FROM relays WHERE uuid = $1")
        .bind(uuid)
        .fetch_optional(pool)
        .await;

    match relay {
        Ok(Some(relay)) => Some(Relay::from_db_relay(relay)),
        _ => None,
    }
}

pub async fn get_all_relays(pool: &PgPool) -> Vec<Relay> {
    let relays = sqlx::query_as("SELECT * FROM relays").fetch_all(pool).await;

    match relays {
        Ok(relays) => relays.into_iter().map(Relay::from_db_relay).collect(),
        _ => vec![],
    }
}

pub struct CreateRelay {
    pub user_npub: String,
    pub name: String,
    pub description: String,
    pub subdomain: String,
    pub custom_domain: String,
    pub instance_type: String,
    pub instance_id: String,
    pub instance_ip: String,
    pub implementation: String,
    pub cloud_provider: String,
    pub write_whitelist: serde_json::Value,
    pub read_whitelist: serde_json::Value,
    pub expires_at: NaiveDateTime,
}

pub async fn create_relay(pool: &PgPool, relay: CreateRelay) -> Result<Relay, Error> {
    let uuid = Uuid::new_v4();
    let db_relay: Relay = sqlx::query_as::<_, Relay>(
        "INSERT INTO relays (uuid, user_npub, name, description, subdomain, custom_domain, instance_type, instance_id, instance_ip, implementation, cloud_provider, write_whitelist, read_whitelist, created_at, updated_at, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
         RETURNING *",
    )
    .bind(uuid.to_string())
    .bind(relay.user_npub.clone())
    .bind(relay.name.clone())
    .bind(relay.description.clone())
    .bind(relay.subdomain.clone())
    .bind(relay.custom_domain.clone())
    .bind(relay.instance_type.clone())
    .bind(relay.instance_id.clone())
    .bind(relay.instance_ip.clone())
    .bind(relay.implementation.clone())
    .bind(relay.cloud_provider.clone())
    .bind(Json(relay.write_whitelist.clone()))
    .bind(Json(relay.read_whitelist.clone()))
    .bind(chrono::Local::now().naive_utc())
    .bind(chrono::Local::now().naive_utc())
    .bind(relay.expires_at.clone())
    .fetch_one(pool)
    .await?;

    Ok(Relay::from_db_relay(db_relay))
}

pub struct UpdateRelay {
    pub name: String,
    pub description: String,
    pub write_whitelist: serde_json::Value,
    pub read_whitelist: serde_json::Value,
}

pub async fn update_relay(
    pool: &PgPool,
    uuid: String,
    update_relay: UpdateRelay,
) -> Result<Relay, Error> {
    let db_relay: Relay = sqlx::query_as::<_, Relay>(
        "UPDATE relays
         SET name = $1, description = $2, write_whitelist = $3, read_whitelist = $4
         WHERE uuid = $5
         RETURNING *",
    )
    .bind(update_relay.name)
    .bind(update_relay.description)
    .bind(Json(update_relay.write_whitelist.clone())) // Using Json type to properly serialize the JSON data
    .bind(Json(update_relay.read_whitelist.clone())) // Using Json type to properly serialize the JSON data
    .bind(uuid.to_string())
    .fetch_one(pool)
    .await?;

    Ok(Relay::from_db_relay(db_relay))
}

#[cfg(test)]
mod tests {
    use crate::{
        models::user::User, repositories::user_repository::create_user,
        util::generators::generate_random_string,
    };

    use super::*;
    use std::env;

    async fn create_test_pool() -> PgPool {
        dotenvy::dotenv().ok();

        let db_url = env::var("DATABASE_URL").expect("TEST_DATABASE_URL must be set to run tests");
        let pool = PgPool::connect(&db_url)
            .await
            .expect("Failed to create test pool");
        clean_up_data(&pool).await;
        pool
    }

    async fn create_test_user() -> String {
        let pool = create_test_pool().await;
        let user_npub = generate_random_string(16).await;

        let user = create_user(&pool, &user_npub)
            .await
            .expect("Failed to create user");

        user.npub
    }

    async fn clean_up_data(pool: &PgPool) {
        sqlx::query!("DELETE FROM relays WHERE uuid != null")
            .execute(pool)
            .await
            .expect("Failed to clean up relays");
        sqlx::query!("DELETE FROM users WHERE npub != null")
            .execute(pool)
            .await
            .expect("Failed to clean up users");
    }

    #[tokio::test]
    async fn test_create_and_get_relay() {
        let pool = create_test_pool().await;

        let relay_name = "Test Relay";
        let relay_description = "This is a test relay";
        let user_npub = create_test_user().await;

        // Test create_relay function
        let relay = CreateRelay {
            user_npub: user_npub,
            name: relay_name.to_string(),
            description: relay_description.to_string(),
            subdomain: generate_random_string(10).await,
            custom_domain: generate_random_string(10).await,
            instance_type: "Type A".to_string(),
            instance_id: generate_random_string(10).await,
            instance_ip: generate_random_string(10).await,
            implementation: "Some implementation".to_string(),
            cloud_provider: "Cloud A".to_string(),
            write_whitelist: json!({"key": "value"}),
            read_whitelist: json!({"key": "value"}),
            expires_at: chrono::Local::now().naive_utc(),
        };
        let created_relay = create_relay(&pool, relay)
            .await
            .expect("Failed to create relay");

        // Test get_relay function
        let retrieved_relay = get_relay(&pool, created_relay.uuid)
            .await
            .expect("Failed to retrieve relay");
        assert_eq!(retrieved_relay.name, relay_name);
        assert_eq!(retrieved_relay.description, relay_description);
    }

    #[tokio::test]
    async fn test_update_relay() {
        let pool = create_test_pool().await;
        let relay_name = "Test Relay";
        let relay_description = "This is a test relay";
        let relay_user_npub = create_test_user().await;

        // Create a relay to update
        let relay = CreateRelay {
            user_npub: relay_user_npub,
            name: relay_name.to_string(),
            description: relay_description.to_string(),
            subdomain: generate_random_string(10).await,
            custom_domain: generate_random_string(10).await,
            instance_type: "Type A".to_string(),
            instance_id: generate_random_string(10).await,
            instance_ip: generate_random_string(10).await,
            implementation: "Some implementation".to_string(),
            cloud_provider: "Cloud A".to_string(),
            write_whitelist: json!({"key": "value"}),
            read_whitelist: json!({"key": "value"}),
            expires_at: chrono::Local::now().naive_utc(),
        };
        let created_relay = create_relay(&pool, relay)
            .await
            .expect("Failed to create relay");

        // Test update_relay function
        let updated_name = "Updated Relay Name";
        let updated_description = "This is an updated relay";
        let updated_write_whitelist = json!({"updated_key": "updated_value"});
        let updated_read_whitelist = json!({"updated_key": "updated_value"});

        let updated_relay = UpdateRelay {
            name: updated_name.to_string(),
            description: updated_description.to_string(),
            write_whitelist: updated_write_whitelist,
            read_whitelist: updated_read_whitelist,
        };

        let updated_relay = update_relay(&pool, created_relay.uuid, updated_relay)
            .await
            .expect("Failed to update relay");

        assert_eq!(updated_relay.name, updated_name);
        assert_eq!(updated_relay.description, updated_description);
    }
}
