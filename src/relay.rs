use super::cloud_provider::{CloudProvider, InstanceType};
use actix_web::{web, HttpResponse, Responder};
use crate::{
    cloud_provider::{launch_instance, LaunchCloudInstance},
    user::UserRepository, middleware::AuthorizationService,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Relay {
    pub uuid: String,
    pub user_npub: String,
    pub name: String,
    pub description: String,
    pub subdomain: String,
    pub custom_domain: String,
    pub instance_type: InstanceType,
    pub instance_id: String,
    pub instance_ip: String,
    pub implementation: RelayImplementation,
    pub cloud_provider: CloudProvider,
    pub write_whitelist: serde_json::Value,
    pub read_whitelist: serde_json::Value,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub expires_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

impl Relay {
    pub fn from_db_relay(relay: Relay) -> Self {
        Relay {
            uuid: relay.uuid,
            user_npub: relay.user_npub,
            name: relay.name,
            description: relay.description,
            subdomain: relay.subdomain,
            custom_domain: relay.custom_domain,
            instance_type: relay.instance_type,
            instance_id: relay.instance_id,
            instance_ip: relay.instance_ip,
            implementation: relay.implementation,
            cloud_provider: relay.cloud_provider,
            write_whitelist: relay.write_whitelist,
            read_whitelist: relay.read_whitelist,
            created_at: relay.created_at,
            updated_at: relay.updated_at,
            expires_at: relay.expires_at,
            deleted_at: relay.deleted_at,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type, Clone, Copy)]
#[sqlx(type_name = "relay_implementation", rename_all = "lowercase")]
pub enum RelayImplementation {
    Strfry,
    NostrRelayRs,
    Nostream,
}

impl RelayImplementation {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelayImplementation::Strfry => "strfry",
            RelayImplementation::NostrRelayRs => "nostrrelayrs",
            RelayImplementation::Nostream => "nostream",
        }
    }
}

pub struct CreateRelay {
    pub user_npub: String,
    pub relay_order_uuid: String,
    pub name: String,
    pub description: String,
    pub subdomain: String,
    pub custom_domain: String,
    pub instance_type: InstanceType,
    pub instance_id: String,
    pub instance_ip: String,
    pub implementation: RelayImplementation,
    pub cloud_provider: CloudProvider,
    pub write_whitelist: serde_json::Value,
    pub read_whitelist: serde_json::Value,
    pub expires_at: NaiveDateTime,
}

pub struct UpdateRelay {
    pub name: String,
    pub description: String,
    pub write_whitelist: serde_json::Value,
    pub read_whitelist: serde_json::Value,
}

pub struct CreateRelayService {
    pub user_npub: String,
    pub relay_order_uuid: String,
    pub name: String,
    pub description: String,
    pub subdomain: Option<String>,
    pub custom_domain: Option<String>,
    pub instance_type: InstanceType,
    pub implementation: RelayImplementation,
    pub cloud_provider: CloudProvider,
    pub write_whitelist: serde_json::Value,
    pub read_whitelist: serde_json::Value,
    pub expires_at: chrono::NaiveDateTime,
}

#[derive(Clone)]
pub struct RelayRepository {
    pub pool: PgPool,
}

impl RelayRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_one(self: &Self, uuid: String) -> Option<Relay> {
        let relay = sqlx::query_as::<_, Relay>("SELECT * FROM relays WHERE uuid = $1")
            .bind(uuid)
            .fetch_optional(&self.pool)
            .await;

        match relay {
            Ok(Some(relay)) => Some(Relay::from_db_relay(relay)),
            _ => None,
        }
    }

    pub async fn get_one_by_user(self: &Self, uuid: String, npub: String) ->Option<Relay> {
        let relay = sqlx::query_as::<_, Relay>("SELECT * FROM relays WHERE uuid = $1 AND user_npub = $2")
            .bind(uuid)
            .bind(npub)
            .fetch_optional(&self.pool)
            .await;

        match relay {
            Ok(Some(relay)) => Some(Relay::from_db_relay(relay)),
            _ => None,
        }
    }

    pub async fn get_all(self: &Self) -> Vec<Relay> {
        let relays = sqlx::query_as("SELECT * FROM relays")
            .fetch_all(&self.pool)
            .await;

        match relays {
            Ok(relays) => relays.into_iter().map(Relay::from_db_relay).collect(),
            _ => vec![],
        }
    }

    pub async fn get_user_relays(self: &Self, npub: String) -> Vec<Relay> {
        let relays = sqlx::query_as::<_, Relay>("SELECT * FROM relays WHERE user_npub = $1")
            .bind(npub)
            .fetch_all(&self.pool)
            .await;

        match relays {
            Ok(relays) => relays.into_iter().map(Relay::from_db_relay).collect(),
            _ => vec![],
        }
    }

    pub async fn delete(self: &Self, uuid: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM relays WHERE uuid = $1")
            .bind(uuid)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn soft_delete(self: &Self, uuid: String) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE relays SET deleted_at = $1 WHERE uuid = $2")
            .bind(NaiveDateTime::from_timestamp_opt(0, 0))
            .bind(uuid)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn create(self: &Self, relay: CreateRelay) -> Result<Relay, sqlx::Error> {
        let uuid = Uuid::new_v4();
        let db_relay: Relay = sqlx::query_as::<_, Relay>(
            "INSERT INTO relays (uuid, user_npub, relay_order_uuid, name, description, subdomain, custom_domain, instance_type, instance_id, instance_ip, implementation, cloud_provider, write_whitelist, read_whitelist, created_at, updated_at, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::relay_instance_type, $9, $10, $11::relay_implementation, $12::relay_cloud_provider, $13, $14, $15, $16, $17)
            RETURNING *",
        )
        .bind(uuid.to_string())
        .bind(relay.user_npub.clone())
        .bind(relay.relay_order_uuid)
        .bind(relay.name.clone())
        .bind(relay.description.clone())
        .bind(relay.subdomain.clone())
        .bind(relay.custom_domain.clone())
        .bind(relay.instance_type)
        .bind(relay.instance_id.clone())
        .bind(relay.instance_ip.clone())
        .bind(relay.implementation)
        .bind(relay.cloud_provider)
        .bind(Json(relay.write_whitelist.clone()))
        .bind(Json(relay.read_whitelist.clone()))
        .bind(chrono::Local::now().naive_utc())
        .bind(chrono::Local::now().naive_utc())
        .bind(relay.expires_at.clone())
        .fetch_one(&self.pool)
        .await?;

        Ok(Relay::from_db_relay(db_relay))
    }

    pub async fn update(
        self: &Self,
        uuid: String,
        update_relay: UpdateRelay,
    ) -> Result<Relay, sqlx::Error> {
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
        .fetch_one(&self.pool)
        .await?;

        Ok(Relay::from_db_relay(db_relay))
    }
}

pub async fn create_relay_service(
    pool: &PgPool,
    relay: CreateRelayService,
) -> Result<Relay, String> {
    let repo = UserRepository::new(pool.clone());
    if !repo.user_exists(relay.user_npub.clone()).await {
        return Err("User does not exist".to_string());
    }

    let launch = LaunchCloudInstance {
        name: relay.name.clone(),
        image_id: dotenvy::var("STRFRY_AMI").unwrap(),
        instance_type: relay.instance_type,
        implementation: relay.implementation,
    };

    let instance = launch_instance(launch).await;

    match instance {
        Ok(instance) => {
            let create_relay = CreateRelay {
                user_npub: relay.user_npub,
                relay_order_uuid: relay.relay_order_uuid,
                name: relay.name,
                description: relay.description,
                subdomain: relay.subdomain.unwrap_or_default(),
                custom_domain: relay.custom_domain.unwrap_or_default(),
                instance_type: relay.instance_type,
                instance_id: instance.id,
                instance_ip: instance.ip_address,
                implementation: relay.implementation,
                cloud_provider: relay.cloud_provider,
                write_whitelist: relay.write_whitelist,
                read_whitelist: relay.read_whitelist,
                expires_at: relay.expires_at,
            };

            let relay = RelayRepository::new(pool.clone())
                .create(create_relay)
                .await
                .expect("Failed to create relay");

            Ok(relay)
        }
        Err(err) => Err(err),
    }
}

// -----------------------------------------------------------------------------
// Handlers
// -----------------------------------------------------------------------------

pub async fn get_relay_handler(
    _auth: AuthorizationService,
    relay_repo: web::Data<RelayRepository>,
    path: web::Path<String>,
) -> impl Responder {
    let relay = relay_repo.get_one(path.into_inner()).await;

    match relay {
        Some(relay) => HttpResponse::Ok().json(relay),
        None => HttpResponse::NotFound().finish(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cloud_provider::terminate_instance,
        relay_order::{CreateRelayOrder, RelayOrderRepository, RelayOrderStatus},
        util::generate_random_string,
    };

    use super::*;
    use serde_json::json;

    async fn create_test_pool() -> PgPool {
        dotenvy::dotenv().ok();

        let db_url =
            dotenvy::var("DATABASE_URL").expect("TEST_DATABASE_URL must be set to run tests");
        let pool = PgPool::connect(&db_url)
            .await
            .expect("Failed to create test pool");

        pool
    }

    #[tokio::test]
    pub async fn test_create_relay_service() {
        let pool = create_test_pool().await;
        let npub = generate_random_string(16).await;
        let user = UserRepository::new(pool.clone()).create(&npub).await;

        let order = RelayOrderRepository::new(pool.clone())
            .create(CreateRelayOrder {
                user_npub: user.unwrap().npub.clone(),
                amount: 100,
                cloud_provider: CloudProvider::AWS,
                instance_type: InstanceType::AwsT2Nano,
                implementation: RelayImplementation::Strfry,
                hostname: "test.relaying.io".to_string(),
                status: RelayOrderStatus::Pending,
            })
            .await
            .expect("Failed to create relay order");

        let user_npub = &npub;
        let name = "Test Relay".to_string();
        let description = "This is a test relay".to_string();
        let instance_type = InstanceType::AwsT2Nano;
        let implementation = RelayImplementation::Strfry;
        let cloud_provider = CloudProvider::AWS;
        let write_whitelist = json!({"key": "value"});
        let read_whitelist = json!({"key": "value"});
        let expires_at = chrono::Local::now().naive_utc();

        let create_relay = CreateRelayService {
            user_npub: user_npub.clone(),
            relay_order_uuid: order.uuid.clone(),
            name: name.clone(),
            description: description.clone(),
            subdomain: None,
            custom_domain: None,
            instance_type: instance_type,
            implementation: implementation,
            cloud_provider: cloud_provider,
            write_whitelist: write_whitelist.clone(),
            read_whitelist: read_whitelist.clone(),
            expires_at: expires_at.clone(),
        };

        let relay = create_relay_service(&pool, create_relay)
            .await
            .expect("Failed to create relay");

        assert_eq!(relay.name, name);
        assert_eq!(relay.description, description);
        assert_eq!(relay.write_whitelist, write_whitelist);
        assert_eq!(relay.read_whitelist, read_whitelist);

        let terminate_instance = terminate_instance(&relay.instance_id).await;
        assert!(terminate_instance.is_ok());

        let delete_relay = RelayRepository::new(pool.clone()).delete(relay.uuid).await;
        assert!(delete_relay.is_ok());

        let delete_user = UserRepository::new(pool.clone()).delete(&user_npub).await;
        assert!(delete_user.is_ok());
    }

    async fn create_test_user() -> String {
        let pool = create_test_pool().await;
        let user_npub = generate_random_string(16).await;

        let user = UserRepository::new(pool)
            .create(&user_npub)
            .await
            .expect("Failed to create user");

        user.npub
    }

    async fn delete_test_user_and_relay(pool: &PgPool, npub: String, relay: Relay) {
        RelayRepository::new(pool.clone())
            .delete(relay.uuid)
            .await
            .expect("Failed to delete relay");

        UserRepository::new(pool.clone())
            .delete(&npub)
            .await
            .expect("Failed to delete user");
    }

    #[tokio::test]
    async fn test_create_and_get_relay() {
        let pool = create_test_pool().await;

        let relay_name = "Test Relay";
        let relay_description = "This is a test relay";
        let user_npub = create_test_user().await;

        let order = RelayOrderRepository::new(pool.clone())
            .create(CreateRelayOrder {
                user_npub: user_npub.clone(),
                amount: 100,
                cloud_provider: CloudProvider::AWS,
                instance_type: InstanceType::AwsT2Nano,
                implementation: RelayImplementation::Strfry,
                hostname: "test.relaying.io".to_string(),
                status: RelayOrderStatus::Pending,
            })
            .await
            .expect("Failed to create relay order");

        // Test create_relay function
        let relay = CreateRelay {
            user_npub: user_npub.clone(),
            relay_order_uuid: order.uuid,
            name: relay_name.to_string(),
            description: relay_description.to_string(),
            subdomain: generate_random_string(10).await,
            custom_domain: generate_random_string(10).await,
            instance_type: InstanceType::AwsT2Nano,
            instance_id: generate_random_string(10).await,
            instance_ip: generate_random_string(10).await,
            implementation: RelayImplementation::Strfry,
            cloud_provider: CloudProvider::AWS,
            write_whitelist: json!({"key": "value"}),
            read_whitelist: json!({"key": "value"}),
            expires_at: chrono::Local::now().naive_utc(),
        };
        let created_relay = RelayRepository::new(pool.clone())
            .create(relay)
            .await
            .expect("Failed to create relay");

        // Test get_relay function
        let retrieved_relay = RelayRepository::new(pool.clone())
            .get_one(created_relay.uuid)
            .await
            .expect("Failed to retrieve relay");
        assert_eq!(retrieved_relay.name, relay_name);
        assert_eq!(retrieved_relay.description, relay_description);

        delete_test_user_and_relay(&pool, user_npub, retrieved_relay).await;
    }

    #[tokio::test]
    async fn test_update_relay() {
        let pool = create_test_pool().await;
        let relay_name = "Test Relay";
        let relay_description = "This is a test relay";
        let relay_user_npub = create_test_user().await;

        let order = RelayOrderRepository::new(pool.clone())
            .create(CreateRelayOrder {
                user_npub: relay_user_npub.clone(),
                amount: 100,
                cloud_provider: CloudProvider::AWS,
                instance_type: InstanceType::AwsT2Nano,
                implementation: RelayImplementation::Strfry,
                hostname: "test.relaying.io".to_string(),
                status: RelayOrderStatus::Pending,
            })
            .await
            .expect("Failed to create relay order");

        // Create a relay to update
        let relay = CreateRelay {
            user_npub: relay_user_npub.clone(),
            relay_order_uuid: order.uuid,
            name: relay_name.to_string(),
            description: relay_description.to_string(),
            subdomain: generate_random_string(10).await,
            custom_domain: generate_random_string(10).await,
            instance_type: InstanceType::AwsT2Nano,
            instance_id: generate_random_string(10).await,
            instance_ip: generate_random_string(10).await,
            implementation: RelayImplementation::Strfry,
            cloud_provider: CloudProvider::AWS,
            write_whitelist: json!({"key": "value"}),
            read_whitelist: json!({"key": "value"}),
            expires_at: chrono::Local::now().naive_utc(),
        };
        let created_relay = RelayRepository::new(pool.clone())
            .create(relay)
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

        let updated_relay = RelayRepository::new(pool.clone())
            .update(created_relay.uuid, updated_relay)
            .await
            .expect("Failed to update relay");

        assert_eq!(updated_relay.name, updated_name);
        assert_eq!(updated_relay.description, updated_description);

        delete_test_user_and_relay(&pool, relay_user_npub, updated_relay).await;
    }
}
